use redis::RedisResult;

use crate::Stride;

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
    let timestamp_key = format!("chain:{}:timestamp", chain_id);
    let live_tps_key = format!("chain:{}:live_tps", chain_id);

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
        .arg(&live_tps_key)
        .arg(timestamp)
        .arg(tx_count)
        .query::<()>(conn)?;

    redis::cmd("SET")
        .arg(&tps_key)
        .arg(tx_count)
        .query::<()>(conn)?;

    redis::cmd("SET")
        .arg(&height_key)
        .arg(height)
        .query::<()>(conn)?;

    redis::cmd("SET")
        .arg(&timestamp_key)
        .arg(timestamp)
        .query::<()>(conn)?;

    Ok(())
}

pub fn get_latest_height(chain_id: &u64, conn: &mut redis::Connection) -> RedisResult<u64> {
    let height_key = format!("chain:{}:height", chain_id);
    let height = redis::cmd("GET").arg(&height_key).query::<u64>(conn)?;

    Ok(height)
}

pub fn get_latest_timestamp(chain_id: &u64, conn: &mut redis::Connection) -> RedisResult<i64> {
    let timestamp_key = format!("chain:{}:timestamp", chain_id);
    let timestamp = redis::cmd("GET").arg(&timestamp_key).query::<i64>(conn)?;

    Ok(timestamp)
}

pub fn get_live_tps(
    chain_id: &u64,
    stride: Stride,
    conn: &mut redis::Connection,
) -> RedisResult<Vec<u64>> {
    let live_tps_key = format!("chain:{}:live_tps", chain_id);
    let latest_timestamp = get_latest_timestamp(chain_id, conn)?;
    let mut stride = stride.stride.unwrap_or(1);
    if stride == 1 {
        stride = 3600; // 1 hr
    } else {
        stride = 600; // 10 in secs
    }
    let raw: Vec<String> = redis::cmd("ZRANGEBYSCORE")
        .arg(&live_tps_key)
        .arg(latest_timestamp.saturating_sub(stride as i64))
        .arg(latest_timestamp)
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

    let tps: Vec<u64> = pairs
        .iter()
        .map(|(member_str, _)| member_str.parse::<u64>().unwrap_or(0))
        .collect();

    Ok(tps)
}

pub fn get_all_chains_live_tps_in_range(
    stride: Stride,
    conn: &mut redis::Connection,
) -> redis::RedisResult<Vec<u64>> {
    let chain_ids: Vec<u64> = redis::cmd("SMEMBERS").arg("chains").query(conn)?;
    let mut all_chains: Vec<u64> = Vec::new();
    for chain_id in chain_ids {
        let chain_sum = get_live_tps(&chain_id, stride.clone(), conn)?;
        all_chains.extend(chain_sum.iter());
    }

    Ok(all_chains)
}

pub fn get_latest_tps(chain_id: &u64, conn: &mut redis::Connection) -> RedisResult<u64> {
    let tps_key = format!("chain:{}:tps", chain_id);
    let tps = redis::cmd("GET").arg(&tps_key).query::<u64>(conn)?;

    Ok(tps)
}

pub fn get_successful_xfers_in_range(
    chain_id: &u64,
    stride: i64,
    conn: &mut redis::Connection,
) -> RedisResult<u64> {
    let successful_key = format!("chain:{}:successful", chain_id);
    let latest_timestamp = get_latest_timestamp(chain_id, conn)?;
    let raw: Vec<String> = redis::cmd("ZRANGEBYSCORE")
        .arg(&successful_key)
        .arg(latest_timestamp.saturating_sub(stride))
        .arg(latest_timestamp)
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
    stride: i64,
    conn: &mut redis::Connection,
) -> redis::RedisResult<u64> {
    let chain_ids: Vec<u64> = redis::cmd("SMEMBERS").arg("chains").query(conn)?;
    let mut total_sum = 0u64;

    for chain_id in chain_ids {
        let chain_sum = get_successful_xfers_in_range(&chain_id, stride, conn)?;
        total_sum += chain_sum;
    }

    Ok(total_sum)
}

pub fn get_all_chains_tps_in_range(conn: &mut redis::Connection) -> redis::RedisResult<u64> {
    let chain_ids: Vec<u64> = redis::cmd("SMEMBERS").arg("chains").query(conn)?;
    let mut total_sum = 0u64;
    for chain_id in chain_ids {
        let chain_sum = get_latest_tps(&chain_id, conn)?;
        total_sum += chain_sum;
    }

    Ok(total_sum)
}
