use bee_core::{Columns, Error, Promise, Row, State, ToData};

#[datasource]
pub fn filesystem(promise: &mut Promise<Filesystem>) -> Result<(), Error> {
    promise.commit(Filesystem {
        name: "dev".to_owned(),
        mount_on: "/dev".to_owned(),
        total_bytes: 1024,
        used_bytes: 1024,
        free_bytes: 1024,
    })?;
    Ok(())
}

#[derive(Data)]
pub struct Filesystem {
    pub name: String,
    pub mount_on: String,
    pub total_bytes: i64,
    pub used_bytes: i64,
    pub free_bytes: i64,
}

#[test]
fn test() {
    let fs = Filesystem {
        name: "dev".to_owned(),
        mount_on: "/dev".to_owned(),
        total_bytes: 1024,
        used_bytes: 1024,
        free_bytes: 1024,
    };

    assert_eq!(
        columns![
            String: "name", 
            String: "mount_on", 
            Integer: "total_bytes", 
            Integer: "used_bytes", 
            Integer: "free_bytes"
        ],
        Filesystem::columns()
    );

    assert_eq!(
        row!["dev","/dev", 1024,1024,1024],
        fs.to_row()
    );
}
