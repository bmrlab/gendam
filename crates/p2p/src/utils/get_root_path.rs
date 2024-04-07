use p2p_block::SpaceblockRequest;

pub fn get_root_path(files: &[SpaceblockRequest]) -> String {
    let mut paths = files[0].path.clone();

    for i in 1..files.len() {
        // 取paths和每个文件的path的公共前缀
        paths = longest_common_prefix(&paths, &files[i].path);
    }

    paths
}

fn longest_common_prefix(a: &str, b: &str) -> String {
    let length = a.len().min(b.len());
    let mut prefix = String::new();

    for i in 0..length {
        if a.chars().nth(i).unwrap() == b.chars().nth(i).unwrap() {
            prefix.push(a.chars().nth(i).unwrap());
        } else {
            break;
        }
    }

    prefix
}
