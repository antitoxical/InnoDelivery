pub fn fake_phone() -> String {
    use fake::Fake;
    use fake::faker::phone_number::en::PhoneNumber;
    PhoneNumber().fake()
}

pub fn fake_email() -> String {
    use fake::Fake;
    use fake::faker::internet::en::SafeEmail;
    SafeEmail().fake()
}
