#[derive(Debug,Clone,Eq,PartialEq)]
pub enum SchedulingPolicy {
    RoundRobin,
    ShortestConnectionFirst,
    Unknown(u8),
}


impl From<u8> for SchedulingPolicy{
    fn from(value: u8) -> Self {

        match value{
            0 => SchedulingPolicy::ShortestConnectionFirst,
            1 => SchedulingPolicy::RoundRobin,
            _ => SchedulingPolicy::Unknown(value),
        }

    }

}

impl From<SchedulingPolicy> for u8{

    fn from(value: SchedulingPolicy) -> Self{

        match value{
            SchedulingPolicy::ShortestConnectionFirst => 0,
            SchedulingPolicy::RoundRobin => 1,
            SchedulingPolicy::Unknown(value) => value,
        }

    }

}

#[cfg(test)]

mod tests{
    use super::*;

    #[test]
    pub fn test_scheduling_policy1(){

        let code = SchedulingPolicy::from(0);
        assert_eq!(SchedulingPolicy::ShortestConnectionFirst, code);
        let code = SchedulingPolicy::from(1);
        assert_eq!(SchedulingPolicy::RoundRobin, code);
        let code = SchedulingPolicy::from(2);
        assert_eq!(SchedulingPolicy::Unknown(2),code);

        let code = 0;
        assert_eq!(SchedulingPolicy::ShortestConnectionFirst,code.into());
        let code = 1;
        assert_eq!(SchedulingPolicy::RoundRobin,code.into());
        let code = 2;
        assert_eq!(SchedulingPolicy::Unknown(2),code.into());

    }
}