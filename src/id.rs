pub fn create_id() -> String {
    use crate::dict::*;
    use rand::seq::SliceRandom;

    let mut rng = rand::thread_rng();
    return DICT_ADJ.choose(&mut rng).unwrap().to_string()
        + &DICT_NOUN.choose(&mut rng).unwrap().to_string();
}
