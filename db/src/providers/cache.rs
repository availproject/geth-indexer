use std::{
    collections::HashSet,
    time::{SystemTime, UNIX_EPOCH},
};

use redis::RedisResult;

use crate::{unix_ms_to_ist, Stride, Tx, Type};

pub fn add_block(
    chain_id: &u64,
    timestamp: i64,
    successful_xfers: u64,
    total_xfers: u64,
    total_native_transfers: u64,
    total_x_chain_transfers: u64,
    tx_count: u64,
    height: u64,
    conn: &mut redis::Connection,
) -> RedisResult<()> {
    // analytics/volume keys
    let successful_key = format!("chain:{}:successful", chain_id);
    let total_key = format!("chain:{}:total", chain_id);

    // TPS keys
    let tps_key = format!("chain:{}:tps", chain_id);
    let x_chain_tps_key = format!("chain:{}:xtps", chain_id);
    let native_chain_tps_key = format!("chain:{}:ntps", chain_id);
    let live_tps_key = format!("chain:{}:live_tps", chain_id);

    // blocks keys
    let height_key = format!("chain:{}:height", chain_id);
    let timestamp_key = format!("chain:{}:timestamp", chain_id);

    // tx keys
    let total_native_txns_key = format!("chain:{}:total_native", chain_id);
    let total_x_chain_txns_key = format!("chain:{}:total_x_chain", chain_id);

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

    redis::cmd("ZADD")
        .arg(&total_native_txns_key)
        .arg(timestamp)
        .arg(total_native_transfers)
        .query::<()>(conn)?;

    redis::cmd("ZADD")
        .arg(&total_x_chain_txns_key)
        .arg(timestamp)
        .arg(total_x_chain_transfers)
        .query::<()>(conn)?;

    redis::cmd("SET")
        .arg(&tps_key)
        .arg(tx_count)
        .query::<()>(conn)?;

    redis::cmd("SET")
        .arg(&x_chain_tps_key)
        .arg(total_x_chain_transfers)
        .query::<()>(conn)?;

    redis::cmd("SET")
        .arg(&native_chain_tps_key)
        .arg(total_native_transfers)
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
    tx_type: Type,
    conn: &mut redis::Connection,
) -> RedisResult<Vec<Entry>> {
    let latest_timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("SystemTime before UNIX EPOCH!")
        .as_secs() as i64;

    let mut stride = stride.stride.unwrap_or(1);
    if stride == 1 {
        stride = 3600;
    } else {
        stride = 600;
    }

    let raw = if let Some(Tx::CrossChain) = tx_type.tx_type {
        let total_x_chain_txns_key = format!("chain:{}:total_x_chain", chain_id);
        let raw: Vec<String> = redis::cmd("ZRANGEBYSCORE")
            .arg(&total_x_chain_txns_key)
            .arg(latest_timestamp.saturating_sub(stride as i64))
            .arg(latest_timestamp)
            .arg("WITHSCORES")
            .query(conn)?;
        raw
    } else if let Some(Tx::Native) = tx_type.tx_type {
        let total_x_chain_txns_key = format!("chain:{}:total_native", chain_id);
        let raw: Vec<String> = redis::cmd("ZRANGEBYSCORE")
            .arg(&total_x_chain_txns_key)
            .arg(latest_timestamp.saturating_sub(stride as i64))
            .arg(latest_timestamp)
            .arg("WITHSCORES")
            .query(conn)?;
        raw
    } else if let Some(Tx::All) = tx_type.tx_type {
        let live_tps_key = format!("chain:{}:live_tps", chain_id);
        let raw: Vec<String> = redis::cmd("ZRANGEBYSCORE")
            .arg(&live_tps_key)
            .arg(latest_timestamp.saturating_sub(stride as i64))
            .arg(latest_timestamp)
            .arg("WITHSCORES")
            .query(conn)?;
        raw
    } else {
        return Ok(Vec::new());
    };

    let mut pairs: Vec<(String, f64)> = Vec::new();
    for chunk in raw.chunks_exact(2) {
        let member_str = chunk[0].clone();
        let score_str = chunk[1].clone();

        if let Ok(score_val) = score_str.parse::<f64>() {
            pairs.push((member_str, score_val));
        }
    }

    let mut tps_pairs: Vec<Entry> = Vec::new();
    for (member_str, score) in &pairs {
        let timestamp = *score as i64;
        let ist_day = unix_ms_to_ist(timestamp);
        if let Ok(value) = member_str.parse::<u64>() {
            tps_pairs.push((value, timestamp, ist_day));
        }
    }

    tps_pairs.sort_by(|a, b| a.2.cmp(&b.2));

    Ok(tps_pairs)
}

pub fn get_all_chains_live_tps_in_range(
    stride: Stride,
    tx_type: Type,
    conn: &mut redis::Connection,
) -> redis::RedisResult<Vec<Entry>> {
    let chain_ids: Vec<u64> = redis::cmd("SMEMBERS").arg("chains").query(conn)?;
    let mut all_chains: Vec<Vec<Entry>> = Vec::new();

    for chain_id in chain_ids {
        match get_live_tps(&chain_id, stride.clone(), tx_type.clone(), conn) {
            Ok(chain_live_tps) => {
                all_chains.push(chain_live_tps);
            }
            Err(e) => {
                return Err(e);
            }
        }
    }

    Ok(accumulate_along_longest_chain(all_chains))
}

pub fn get_latest_tps(
    chain_id: &u64,
    tx_type: Type,
    conn: &mut redis::Connection,
) -> RedisResult<u64> {
    let tps_key = if let Some(Tx::CrossChain) = tx_type.tx_type {
        format!("chain:{}:xtps", chain_id)
    } else if let Some(Tx::Native) = tx_type.tx_type {
        format!("chain:{}:ntps", chain_id)
    } else if let Some(Tx::All) = tx_type.tx_type {
        format!("chain:{}:tps", chain_id)
    } else {
        return Ok(0);
    };

    let tps = redis::cmd("GET").arg(&tps_key).query::<u64>(conn)?;

    Ok(tps)
}

pub fn get_successful_xfers_in_range(
    chain_id: &u64,
    stride: i64,
    latest_timestamp: i64,
    tx_type: Type,
    conn: &mut redis::Connection,
) -> RedisResult<u64> {
    let raw = if let Some(Tx::CrossChain) = tx_type.tx_type {
        let total_x_chain_txns_key = format!("chain:{}:total_x_chain", chain_id);
        let raw: Vec<String> = redis::cmd("ZRANGEBYSCORE")
            .arg(&total_x_chain_txns_key)
            .arg(latest_timestamp.saturating_sub(stride))
            .arg(latest_timestamp)
            .arg("WITHSCORES")
            .query(conn)?;
        raw
    } else if let Some(Tx::Native) = tx_type.tx_type {
        let total_x_chain_txns_key = format!("chain:{}:total_native", chain_id);
        let raw: Vec<String> = redis::cmd("ZRANGEBYSCORE")
            .arg(&total_x_chain_txns_key)
            .arg(latest_timestamp.saturating_sub(stride))
            .arg(latest_timestamp)
            .arg("WITHSCORES")
            .query(conn)?;
        raw
    } else if let Some(Tx::All) = tx_type.tx_type {
        let successful_key = format!("chain:{}:successful", chain_id);
        let raw: Vec<String> = redis::cmd("ZRANGEBYSCORE")
            .arg(&successful_key)
            .arg(latest_timestamp.saturating_sub(stride as i64))
            .arg(latest_timestamp)
            .arg("WITHSCORES")
            .query(conn)?;
        raw
    } else {
        return Ok(0);
    };

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
    tx_type: Type,
    conn: &mut redis::Connection,
) -> redis::RedisResult<u64> {
    let chain_ids: Vec<u64> = redis::cmd("SMEMBERS").arg("chains").query(conn)?;
    let mut total_sum = 0u64;

    for chain_id in chain_ids {
        let chain_sum = get_successful_xfers_in_range(
            &chain_id,
            stride,
            latest_timestamp,
            tx_type.clone(),
            conn,
        )?;
        total_sum += chain_sum;
    }

    Ok(total_sum)
}

pub fn get_all_chains_tps_in_range(
    tx_type: Type,
    conn: &mut redis::Connection,
) -> redis::RedisResult<u64> {
    let chain_ids: Vec<u64> = redis::cmd("SMEMBERS").arg("chains").query(conn)?;
    let mut total_sum = 0u64;
    for chain_id in chain_ids {
        let chain_sum = get_latest_tps(&chain_id, tx_type.clone(), conn)?;
        total_sum += chain_sum;
    }

    Ok(total_sum)
}

pub fn accumulate_along_longest_chain(chains: Vec<Vec<Entry>>) -> Vec<Entry> {
    if chains.is_empty() {
        return vec![];
    }
    let base_idx = chains
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| {
            let len_cmp = a.len().cmp(&b.len());
            if len_cmp == std::cmp::Ordering::Equal {
                let a_ts = a.first().map(|x| x.1).unwrap_or(i64::MAX);
                let b_ts = b.first().map(|x| x.1).unwrap_or(i64::MAX);
                b_ts.cmp(&a_ts)
            } else {
                len_cmp
            }
        })
        .map(|(idx, _)| idx)
        .unwrap_or(0);
    let base_chain = &chains[base_idx];
    let mut used_indices: Vec<HashSet<usize>> = chains.iter().map(|_| HashSet::new()).collect();
    let mut result = Vec::with_capacity(base_chain.len());
    for &(base_tps, base_ts, ref stringified_base_timestamp) in base_chain {
        let mut total_tps = base_tps;
        for (chain_idx, chain) in chains.iter().enumerate() {
            if chain_idx == base_idx {
                continue;
            }
            let mut closest: Option<(usize, i64)> = None;
            for (i, &(_, ts, _)) in chain.iter().enumerate() {
                if used_indices[chain_idx].contains(&i) {
                    continue;
                }
                let diff = (ts - base_ts).abs();
                if closest.map_or(true, |(_, best_diff)| diff < best_diff) {
                    closest = Some((i, diff));
                }
            }
            if let Some((i, _)) = closest {
                used_indices[chain_idx].insert(i);
                total_tps += chain[i].0;
            }
        }
        result.push((total_tps, base_ts, stringified_base_timestamp.clone()));
    }
    result
}

type Entry = (u64, i64, String);
