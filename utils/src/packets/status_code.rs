use std::fmt::{Display, Formatter};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StatusCode{
    S404,
    S200,
    Unknown(u8),
}


impl From<u8> for StatusCode{
    fn from(code: u8) -> Self{
        match code{
            1 => StatusCode::S200,
            2 => StatusCode::S404,
            _ => StatusCode::Unknown(code),
        }
    }
}

impl From<StatusCode> for u8{
    fn from(code: StatusCode) -> Self{

        match code{
            StatusCode::S404 => 1,
            StatusCode::S200 => 2,
            StatusCode::Unknown(code) => code,
        }

    }
}

impl Display for StatusCode{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            StatusCode::S404 => write!(f, "404"),
            StatusCode::S200 => write!(f, "200"),
            StatusCode::Unknown(n) => write!(f,"UnknownStatusCode({})",n),
        }
    }
}

#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    pub fn test_status_code1(){

        let code = StatusCode::from(1);
        assert_eq!(StatusCode::S200, code);
        let code = StatusCode::from(2);
        assert_eq!(StatusCode::S404, code);
        let code = StatusCode::from(3);
        assert_eq!(StatusCode::Unknown(3),code);

        let code = 1;
        assert_eq!(StatusCode::S200,code.into());
        let code = 2;
        assert_eq!(StatusCode::S404,code.into());
        let code = 3;
        assert_eq!(StatusCode::Unknown(3),code.into());

    }
}