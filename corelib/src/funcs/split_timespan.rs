use crate::{Result, Value};

#[function]
pub fn split_timespan(input: String) -> Result<u32> {
    let items: Vec<&str> = input.trim().split(":").collect();
    let h = items
        .get(0)
        .map(|val| val.parse::<i32>())
        .unwrap_or(Ok(0))?;
    let m = items
        .get(1)
        .map(|val| val.parse::<i32>())
        .unwrap_or(Ok(0))?;
    let s = items
        .get(2)
        .map(|val| val.parse::<i32>())
        .unwrap_or(Ok(0))?;

    Ok(((h * 3600) + (m * 60) + s) as u32)
}

#[test]
fn test1() {
    let arg = "00:00:06".to_owned();
    let rs = split_timespan(arg).unwrap();
    assert_eq!(6, rs);
}

#[test]
fn test2() {
    let arg = " 01:00:06".to_owned();
    let rs = split_timespan(arg).unwrap();
    assert_eq!(3606, rs);
}

#[test]
fn test3() {
    let arg = " 00:01:06".to_owned();
    let rs = split_timespan(arg).unwrap();
    assert_eq!(66, rs);
}

#[test]
fn test4() {
    let arg = " 01:06".to_owned();
    let rs = split_timespan(arg).unwrap();
    assert_eq!(3960, rs);
}
