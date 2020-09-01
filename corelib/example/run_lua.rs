use std::time::Duration;

fn main() {
    const MAX_CONN: u32 = 100; 
    const MAX_TASK: u32 = 100;

    let script = r#"
        local resp=filesystem();
        while(resp:has_next())
        do
            _request:commit(_next);
        end
    "#;

    async_std::task::block_on(async {
        for _ in 0..MAX_CONN {
            let connection = bee_core::new_connection("lua:agent:default").await.unwrap();

            for _ in 0..MAX_TASK{
                let statement = connection.new_statement(script,Duration::from_secs(2)).await.unwrap();
                let resp = statement.wait().unwrap();
                let mut index = 0;
                for rs in resp {
                    let _ = rs.unwrap();
                    index += 1;
                }
                println!("index: {}", index);
            }
            drop(connection);
        }
    });

    std::thread::sleep(Duration::from_secs(60));
}
