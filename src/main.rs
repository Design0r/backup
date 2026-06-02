use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use anyhow::Result;
use chrono::Local;
use dircpy::copy_dir;

static BACKUPS: &'static [(&str, &str)] = &[(
    r"\\apicnas\Produktion\Pipeline\Apic Studio\apic_studio.db",
    r"\\apicnas\Produktion\Pipeline\Apic Studio\db\apic_studio_{DATE}.db",
)];

fn create_backup(src: &Path, dst: &Path) -> Result<()> {
    if src.is_dir() {
        copy_dir(src, dst)?;
    } else {
        fs::copy(src, dst)?;
    }

    Ok(())
}

fn replace(input: &str) -> String {
    let now = format!("{}", Local::now().format("%y%m%d"));
    input.replace("{DATE}", &now)
}

fn get_file_hash(path: &Path) -> Result<String> {
    let buf = fs::read(path)?;
    let hash = sha256::digest(&buf);
    Ok(hash)
}

fn delete_duplicates(path: &Path) -> Result<()> {
    let mut map = HashMap::<String, Vec<PathBuf>>::new();
    fs::read_dir(path)?.filter_map(|e| e.ok()).for_each(|e| {
        let path = e.path();
        let hash = get_file_hash(&path).unwrap();

        if map.contains_key(&hash) {
            let buf = map.get_mut(&hash).unwrap();
            buf.push(path);
        } else {
            let v = vec![path];
            map.insert(hash, v);
        }
    });

    for (_, v) in map {
        for dupe in &v[1..] {
            println!("removing {dupe:?} as duplicate from {:?}", &v[0]);
            fs::remove_file(&dupe)?;
        }
    }

    Ok(())
}

fn main() {
    BACKUPS
        .into_iter()
        .map(|(a, b)| (replace(a), replace(b)))
        .map(|(a, b)| (PathBuf::from(a), PathBuf::from(b)))
        .enumerate()
        .for_each(|(i, (a, b))| {
            println!("creating backup {}/{}: {b:?}", i + 1, BACKUPS.len());
            create_backup(&a, &b)
                .map_err(|e| format!("failed to create backup: {e}"))
                .unwrap();
            println!("finished backup {}/{}: {b:?}", i + 1, BACKUPS.len());
        });

    let p = PathBuf::from(r"\\apicnas\Produktion\Pipeline\Apic Studio\db\");
    let _ = delete_duplicates(&p);
}
