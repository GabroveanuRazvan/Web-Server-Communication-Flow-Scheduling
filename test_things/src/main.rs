use rand::{thread_rng, Rng};
use utils::config::serialization::save;
use utils::html_prefetch_service::HtmlPrefetchService;

const REQUEST_COUNT: usize = 250;

fn main() {

   let mut prefetch_service = HtmlPrefetchService::new("../benchmarking/raw_dataset");
   prefetch_service.build_prefetch_links().unwrap();

   let links = prefetch_service.get_links();

   let keys: Vec<&String> = links.keys().collect();
   let key_count =  keys.len();

   let mut rng = thread_rng();
   
   let mut requests = Vec::with_capacity(REQUEST_COUNT);

   (0..REQUEST_COUNT).for_each(|_| {

      let random_usize = rng.gen_range(0..key_count);
      
      let key = keys[random_usize].clone();
      
      requests.push(key.clone());
      requests.extend_from_slice(links.get(&key).unwrap());
      
   });
   
   save(requests.clone(),"../benchmarking/requests/prefetch_requests_5_5000.json").unwrap();
   
   
   println!("{:#?}",requests);
   println!("Requests count: {}", requests.len());
}
