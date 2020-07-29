pub fn create_id() -> String {
    use rand::Rng;

    let mut rng = rand::thread_rng();
    return crate::dict::DICT[rng.gen_range(0, 194433)].to_string();
}
