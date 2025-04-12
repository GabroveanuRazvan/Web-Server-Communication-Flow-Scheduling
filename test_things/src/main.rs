use std::thread::sleep;
use std::time::Duration;
use utils::pools::thread_pool::ThreadPool;
#[inline(never)]
fn add(mut num : u128){
    sleep(Duration::from_millis(num as u64));
    loop{
        num = num.wrapping_add(1);

    }

}


fn main(){

    let pool = ThreadPool::new(12);

    for i in 0..12{
        pool.execute(move||{
            add(i);
        })
    }

}

