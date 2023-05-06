use crate::repository::user::{User, UserRepository};

#[derive(Clone)]
pub struct LocalUserRepository<'a> {
    users: Vec<User<'a>>,
}

impl LocalUserRepository<'_> {
    pub fn new() -> LocalUserRepository<'static> {
        return LocalUserRepository { users: Vec::new() };
    }
}

impl UserRepository for LocalUserRepository<'static> {
    fn create<'a, 'b>(&mut self, user: User<'static>) -> Result<User<'a>, &'b str> {
        self.users.push(user.clone());
        Ok(user)
    }

    fn read<'a, 'b>(&self, phone_number: u64) -> Result<Vec<User<'a>>, &'b str> {
        Ok(self
            .users
            .clone()
            .into_iter()
            .filter(|u| u.phone_number == phone_number)
            .collect())
    }

    fn update<'a, 'b>(&mut self, user: User<'static>) -> Result<User<'a>, &'b str> {
        self.users = self
            .users
            .clone()
            .into_iter()
            .map(|u| {
                if u.phone_number == user.phone_number {
                    return user.clone();
                };
                u
            })
            .collect();
        Ok(user)
    }

    fn delete<'a>(&mut self, phone_number: u64) -> Result<Option<User<'a>>, &'a str> {
        let mut user: Option<User> = None;
        self.users = self
            .users
            .clone()
            .into_iter()
            .filter(|u| {
                if u.phone_number == phone_number {
                    user = Some(u.clone());
                };
                u.phone_number != phone_number
            })
            .collect();
        Ok(user)
    }
}
