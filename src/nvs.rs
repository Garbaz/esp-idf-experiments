use esp_idf_svc::nvs::{EspNvs, EspNvsPartition, NvsPartitionId};
use serde::{Deserialize, Serialize};

/// In the ESP's non-volatile store in the namespace `namespace` under the key
/// `key` store the value `x`.
pub fn store<const BUFFER_SIZE: usize, I, T>(
    nvs_partition: EspNvsPartition<I>,
    namespace: &str,
    key: &str,
    x: T,
) -> anyhow::Result<()>
where
    I: NvsPartitionId,
    T: Serialize,
{
    let mut nvs = EspNvs::new(nvs_partition, namespace, true)?;

    let buffer = postcard::to_vec::<_, BUFFER_SIZE>(&x)?;

    nvs.set_raw(key, &buffer)?;

    Ok(())
}

/// From the ESP's non-volatile storage try to load the value in the namespace
/// `namespace` at the key `key`.
pub fn load<const BUFFER_SIZE: usize, I, T>(
    nvs_partition: EspNvsPartition<I>,
    namespace: &str,
    key: &str,
) -> anyhow::Result<Option<T>>
where
    I: NvsPartitionId,
    T: for<'a> Deserialize<'a>,
{
    let nvs = EspNvs::new(nvs_partition, namespace, true)?;

    let buffer: &mut [u8] = &mut [0; BUFFER_SIZE];

    Ok(match nvs.get_raw(key, buffer)? {
        Some(b) => Some(postcard::from_bytes(b)?),
        None => None,
    })
}

pub fn load_or_else_store<const BUFFER_SIZE: usize, I, T, F>(
    nvs_partition: EspNvsPartition<I>,
    namespace: &str,
    key: &str,
    f: F,
) -> anyhow::Result<T>
where
    I: NvsPartitionId,
    T: Serialize + for<'a> Deserialize<'a>,
    F: FnOnce() -> T,
{
    let mut nvs = EspNvs::new(nvs_partition, namespace, true)?;

    let buffer: &mut [u8] = &mut [0; BUFFER_SIZE];

    let x = match nvs.get_raw(key, buffer)? {
        Some(b) => Some(postcard::from_bytes(b)?),
        None => None,
    };

    let x = match x {
        Some(x) => x,
        None => {
            let x = f();
            let buffer = postcard::to_vec::<_, BUFFER_SIZE>(&x)?;
            nvs.set_raw(key, &buffer)?;
            x
        }
    };

    Ok(x)
}
