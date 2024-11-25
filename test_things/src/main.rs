
fn main() ->Result<(),std::io::Error>{

    // mount_tmpfs("mytmpfs",100)?;


    let command = Command::new("sudo mount")
        .args(["-t", "tmpfs", "-o", &format!("size={}M", 110), "tmpfs", "mytmpfs"]);
    let args = command.get_args();

    println!("{:?}", command);

    Ok(())
}
use std::process::Command;
use std::fs;


fn mount_tmpfs(mount_point: &str, size: i32) -> Result<(), std::io::Error> {
    fs::create_dir_all(mount_point)?;
    Command::new("sudo mount")
        .args(["-t", "tmpfs", "-o", &format!("size={}M", size), "tmpfs", mount_point])
        .status()?;
    Ok(())
}

fn umount_tmpfs(mount_point: &str) -> Result<(), std::io::Error> {
    Command::new("umount").arg(mount_point).status()?;
    fs::remove_dir_all(mount_point)?;
    Ok(())
}