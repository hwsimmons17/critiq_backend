#[derive(Clone)]
pub struct User<'a> {
    pub first_name: &'a str,
    pub last_name: &'a str,
    pub phone_number: u64,
    pub is_verified: bool,
}

pub trait UserRepository: Clone + Send + Sync + 'static {
    fn create<'a, 'b>(&mut self, user: User<'static>) -> Result<User<'a>, &'b str>;
    fn read<'a, 'b>(&self, phone_number: u64) -> Result<Vec<User<'a>>, &'b str>;
    fn update<'a, 'b>(&mut self, user: User<'static>) -> Result<User<'a>, &'b str>;
    fn delete<'a>(&mut self, phone_number: u64) -> Result<Option<User<'a>>, &'a str>;
}
