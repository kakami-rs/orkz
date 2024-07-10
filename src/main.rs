use tokio::io::AsyncSeekExt;
use tokio_util::codec::{BytesCodec, FramedRead};
use futures_util::{TryStreamExt, StreamExt};

#[tokio::main]
async fn main() {
    let mut path = dirs_next::home_dir().unwrap();
    path.push("Library/Logs/Rcd".to_string());
    let mut entries = tokio::fs::read_dir(path).await.unwrap();
    let mut path1: Option<std::path::PathBuf> = None;
    let mut path1_size: u64 = 0;
    let mut path2: Option<std::path::PathBuf> = None;
    let mut sec1: std::time::SystemTime = std::time::SystemTime::UNIX_EPOCH;
    let mut sec2: std::time::SystemTime = std::time::SystemTime::UNIX_EPOCH;
    while let Ok(Some(entry)) = entries.next_entry().await {
        println!("{:?}", entry.path());
        // entry最后修改时期
        let metadata = tokio::fs::metadata(entry.path()).await.unwrap();
        let sec = metadata.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH);
        if path1.is_some() {
            if sec > sec1 {
                sec2 = sec1;
                path2 = path1;
                sec1 = sec;
                path1 = Some(entry.path());
                path1_size = metadata.len();
            } else if sec > sec2 {
                sec2 = sec;
                path2 = Some(entry.path());
            }
        } else {
            sec1 = sec;
            path1 = Some(entry.path());
            path1_size = metadata.len();
        }
    }
    println!("{:?} - {}", path1, path1_size);
    println!("{:?}", path2);
    let path1 = {
        if let Some(path) = path1 {
            path
        } else {
            return;
        }
    };

    let mut total_size: usize = 0;
    if path1_size > 1024 * 1024 {
        if let Ok(file) = tokio::fs::File::open(path1).await {
            let mut reader = tokio::io::BufReader::new(file);
            reader.seek(tokio::io::SeekFrom::End(-10240 * 1024)).await.unwrap();
            let mut stream = FramedRead::new(reader, BytesCodec::new())
                .map_ok(|bytes| bytes.freeze());
            while let Some(item) = stream.next().await {
                if let Ok(item) = item {
                    println!("{}", item.len());
                    total_size += item.len();
                }
            }
        }
    }
    println!("total_size: {}", total_size);
}

