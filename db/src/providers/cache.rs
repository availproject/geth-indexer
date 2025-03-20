use std::u64::MIN;

use redis::RedisResult;

use crate::{unix_ms_to_ist, Stride};

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
) -> RedisResult<Vec<(u64, String)>> {
    let live_tps_key = format!("chain:{}:live_tps", chain_id);
    let latest_timestamp = get_latest_timestamp(chain_id, conn)?;

    let mut stride = stride.stride.unwrap_or(1);
    if stride == 1 {
        stride = 3600;
    } else {
        stride = 600;
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

    let mut tps_pairs: Vec<(u64, String)> = Vec::new();
    for (member_str, score) in &pairs {
        let timestamp = *score as i64;
        let ist_day = unix_ms_to_ist(timestamp);
        if let Ok(value) = member_str.parse::<u64>() {
            tps_pairs.push((value, ist_day));
        }
    }

    Ok(tps_pairs)
}

pub fn get_all_chains_live_tps_in_range(
    stride: Stride,
    conn: &mut redis::Connection,
) -> redis::RedisResult<Vec<(u64, String)>> {
    let chain_ids: Vec<u64> = redis::cmd("SMEMBERS").arg("chains").query(conn)?;
    let mut all_chains: Vec<Vec<(u64, String)>> = Vec::new();
    let mut lowest_size = usize::MAX; // Set to max initially

    for chain_id in chain_ids {
        match get_live_tps(&chain_id, stride.clone(), conn) {
            Ok(chain_sum) => {
                lowest_size = std::cmp::min(lowest_size, chain_sum.len());
                all_chains.push(chain_sum);
            }
            Err(e) => {
                return Err(e);
            }
        }
    }

    tracing::info!(" all_chain {:?}", all_chains);

    let mut final_chain_live_tps = Vec::with_capacity(lowest_size);

    for i in 0..lowest_size {
        let (value, timestamp) = all_chains
            .iter()
            .filter_map(|chain| chain.get(i))
            .fold((0, None), |(sum, _), &(val, ref ts)| {
                (sum + val, Some(ts.clone()))
            });

        tracing::info!("value:{}, timestamp:{:?}", value, timestamp);
        if let Some(ts) = timestamp {
            final_chain_live_tps.push((value, ts));
        }
    }

    Ok(final_chain_live_tps)
}

pub fn get_latest_tps(chain_id: &u64, conn: &mut redis::Connection) -> RedisResult<u64> {
    let tps_key = format!("chain:{}:tps", chain_id);
    let tps = redis::cmd("GET").arg(&tps_key).query::<u64>(conn)?;

    Ok(tps)
}

pub fn get_successful_xfers_in_range(
    chain_id: &u64,
    stride: i64,
    latest_timestamp: i64,
    conn: &mut redis::Connection,
) -> RedisResult<u64> {
    let successful_key = format!("chain:{}:successful", chain_id);
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
    latest_timestamp: i64,
    conn: &mut redis::Connection,
) -> redis::RedisResult<u64> {
    let chain_ids: Vec<u64> = redis::cmd("SMEMBERS").arg("chains").query(conn)?;
    let mut total_sum = 0u64;

    for chain_id in chain_ids {
        let chain_sum = get_successful_xfers_in_range(&chain_id, stride, latest_timestamp, conn)?;
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
