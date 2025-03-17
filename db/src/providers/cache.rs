use redis::RedisResult;

pub fn add_block(
    chain_id: &u64,
    timestamp: i64,
    successful_xfers: u64,
    total_xfers: u64,
    tx_count: u64,
    height: u64,
    conn: &mut redis::Connection,
) -> RedisResult<()> {
    let successful_key = format!("chain:{}:successful", chain_id);
    let total_key = format!("chain:{}:total", chain_id);
    let tps_key = format!("chain:{}:tps", chain_id);
    let height_key = format!("chain:{}:height", chain_id);

    redis::cmd("SADD")
        .arg("chains")
        .arg(chain_id.to_string())
        .query::<()>(conn)?;

    redis::cmd("ZADD")
        .arg(&successful_key)
        .arg(timestamp)
        .arg(successful_xfers)
        .query::<()>(conn)?;

    redis::cmd("ZADD")
        .arg(&total_key)
        .arg(timestamp)
        .arg(total_xfers)
        .query::<()>(conn)?;

    redis::cmd("ZADD")
        .arg(&tps_key)
        .arg(timestamp)
        .arg(tx_count)
        .query::<()>(conn)?;

    redis::cmd("SET")
        .arg(&height_key)
        .arg(height)
        .query::<()>(conn)?;

    Ok(())
}

pub fn get_latest_height(chain_id: &u64, conn: &mut redis::Connection) -> RedisResult<u64> {
    let height_key = format!("chain:{}:height", chain_id);
    let height = redis::cmd("GET").arg(&height_key).query::<u64>(conn)?;

    Ok(height)
}

pub fn get_latest_tps_in_range(
    chain_id: &u64,
    start_ts: i64,
    end_ts: i64,
    conn: &mut redis::Connection,
) -> RedisResult<u64> {
    let tps_key = format!("chain:{}:tps", chain_id);
    let raw: Vec<String> = redis::cmd("ZRANGEBYSCORE")
        .arg(&tps_key)
        .arg(start_ts)
        .arg(end_ts)
        .arg("WITHSCORES")
        .query(conn)?;

    let mut pairs: Vec<(String, f64)> = Vec::new();
    for chunk in raw.chunks_exact(2) {
        let member_str = chunk[0].clone();
        let score_str = chunk[1].clone();

        if let Ok(score_val) = score_str.parse::<f64>() {
            pairs.push((member_str, score_val));
        }
    }

    let sum: u64 = pairs
        .iter()
        .map(|(member_str, _)| member_str.parse::<u64>().unwrap_or(0))
        .sum();

    Ok(sum)
}

pub fn get_successful_xfers_in_range(
    chain_id: &u64,
    start_ts: i64,
    end_ts: i64,
    conn: &mut redis::Connection,
) -> RedisResult<u64> {
    let successful_key = format!("chain:{}:successful", chain_id);
    let raw: Vec<String> = redis::cmd("ZRANGEBYSCORE")
        .arg(&successful_key)
        .arg(start_ts)
        .arg(end_ts)
        .arg("WITHSCORES")
        .query(conn)?;

    let mut pairs: Vec<(String, f64)> = Vec::new();
    for chunk in raw.chunks_exact(2) {
        let member_str = chunk[0].clone();
        let score_str = chunk[1].clone();

        if let Ok(score_val) = score_str.parse::<f64>() {
            pairs.push((member_str, score_val));
        }
    }

    let sum: u64 = pairs
        .iter()
        .map(|(member_str, _)| member_str.parse::<u64>().unwrap_or(0))
        .sum();

    Ok(sum)
}

pub fn get_all_chains_success_xfers_in_range(
    start_ts: i64,
    end_ts: i64,
    conn: &mut redis::Connection,
) -> redis::RedisResult<u64> {
    let chain_ids: Vec<u64> = redis::cmd("SMEMBERS").arg("chains").query(conn)?;

    let mut total_sum = 0u64;

    for chain_id in chain_ids {
        let chain_sum = get_successful_xfers_in_range(&chain_id, start_ts, end_ts, conn)?;
        total_sum += chain_sum;
    }

    Ok(total_sum)
}

pub fn get_all_chains_tps_in_range(
    start_ts: i64,
    end_ts: i64,
    conn: &mut redis::Connection,
) -> redis::RedisResult<u64> {
    let chain_ids: Vec<u64> = redis::cmd("SMEMBERS").arg("chains").query(conn)?;

    let mut total_sum = 0u64;

    for chain_id in chain_ids {
        let chain_sum = get_latest_tps_in_range(&chain_id, start_ts, end_ts, conn)?;
        total_sum += chain_sum;
    }

    Ok(total_sum)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::DatabaseConnections;
    use redis::RedisResult;
    use tokio::sync::Mutex;

    fn flush_redis(conn: &mut redis::Connection) -> RedisResult<()> {
        redis::cmd("FLUSHALL").query::<()>(conn)?;
        Ok(())
    }

    #[tokio::test]
    async fn test_all_chains_data_inserts_and_queries() -> RedisResult<()> {
        let conns = Arc::new(Mutex::new(DatabaseConnections::init_redis()));

        {
            let mut redis_conn = conns.lock().await;
            flush_redis(&mut redis_conn)?;
        }

        let now = 1_000_000_000i64;

        {
            let mut redis_conn = conns.lock().await;
            let ts_a = vec![
                now - 1800, // 30 min ago
                now - 1200, // 20 min ago
                now - 600,  // 10 min ago
                now,        // current
            ];
            let data_a = vec![(10, 12), (20, 25), (30, 35), (40, 45)];

            for (ts, (succ, tot)) in ts_a.iter().zip(data_a) {
                add_block(&1001, *ts, succ, tot, 0, 0, &mut redis_conn)?;
            }
        }

        {
            let mut redis_conn = conns.lock().await;
            let ts_b = vec![
                now - 1500, // 25 min ago
                now - 900,  // 15 min ago
                now - 300,  // 5 min ago
                now,
            ];
            let data_b = vec![(5, 10), (15, 20), (25, 30), (50, 60)];

            for (ts, (succ, tot)) in ts_b.iter().zip(data_b) {
                add_block(&1002, *ts, succ, tot, 0, 0, &mut redis_conn)?;
            }
        }

        {
            let mut redis_conn = conns.lock().await;

            let last_20_min_start = now - 1200; // 20 min window
            let chain_a_20min =
                get_successful_xfers_in_range(&1001, last_20_min_start, now, &mut redis_conn)?;
            assert_eq!(chain_a_20min, 20 + 30 + 40); // 20-min window hits 3 data points

            let chain_b_20min =
                get_successful_xfers_in_range(&1002, last_20_min_start, now, &mut redis_conn)?;
            assert_eq!(chain_b_20min, 15 + 25 + 50);

            let all_20min =
                get_all_chains_success_xfers_in_range(last_20_min_start, now, &mut redis_conn)?;
            assert_eq!(all_20min, chain_a_20min + chain_b_20min);
        }

        let redis_conn = conns.clone();

        tokio::spawn(async move {
            let mut redis_conn = redis_conn.lock().await;

            let last_30_min_start = now - 1800;
            let a_30min =
                get_successful_xfers_in_range(&1001, last_30_min_start, now, &mut redis_conn)
                    .unwrap();
            assert_eq!(a_30min, 10 + 20 + 30 + 40);

            let b_30min =
                get_successful_xfers_in_range(&1002, last_30_min_start, now, &mut redis_conn)
                    .unwrap();
            assert_eq!(b_30min, 5 + 15 + 25 + 50);
        });

        Ok(())
    }
}
