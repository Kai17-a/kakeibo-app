use kakeibo_app::auth::challenge_store::{CHALLENGE_TTL_SECONDS, ChallengeStore, MAX_PENDING};

const NOW: i64 = 1_000_000;

#[test]
fn a_challenge_can_be_taken_only_once() {
    let store = ChallengeStore::default();
    let id = store.insert("state", NOW).unwrap();

    assert_eq!(store.take(&id, NOW), Some("state"));
    assert_eq!(store.take(&id, NOW), None);
}

#[test]
fn an_expired_challenge_cannot_be_taken() {
    let store = ChallengeStore::default();
    let id = store.insert("state", NOW).unwrap();

    assert_eq!(store.take(&id, NOW + CHALLENGE_TTL_SECONDS), None);
}

#[test]
fn a_flood_of_challenges_never_blocks_new_ones() {
    let store = ChallengeStore::default();
    for index in 0..MAX_PENDING * 2 {
        store.insert(index, NOW).unwrap();
    }

    // Anyone can ask for a login challenge, so a full store must not lock everyone out.
    let id = store.insert(usize::MAX, NOW).unwrap();
    assert_eq!(store.take(&id, NOW), Some(usize::MAX));
}

#[test]
fn the_oldest_challenge_makes_room_when_the_store_is_full() {
    let store = ChallengeStore::default();
    let oldest = store.insert(0, NOW).unwrap();
    let ids: Vec<_> = (1..MAX_PENDING)
        .map(|index| store.insert(index, NOW + 1).unwrap())
        .collect();

    store.insert(MAX_PENDING, NOW + 2).unwrap();

    assert_eq!(store.take(&oldest, NOW + 2), None);
    assert_eq!(store.take(&ids[0], NOW + 2), Some(1));
}
