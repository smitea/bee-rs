use std::time::Duration;

fn main() {
    const MAX_CONN: u32 = 1000; 

    let script = r#"
        SELECT * FROM host_basic()
    "#;

    async_std::task::block_on(async {
        for _ in 0..MAX_CONN {
            let connection = bee_core::new_connection("sqlite:agent:default").await.unwrap();
            let statement = connection.new_statement(script,Duration::from_secs(2)).await.unwrap();
            let resp = statement.wait().unwrap();
            let mut index = 0;
            for rs in resp {
                let _ = rs.unwrap();
                index += 1;
            }
            println!("index: {}", index);
        }
    });
}
