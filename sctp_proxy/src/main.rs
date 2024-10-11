
use utils::sctp_api::SctpEventSubscribe;

fn main() {

    let x = SctpEventSubscribe::new();

    println!("{x:?}");
}
