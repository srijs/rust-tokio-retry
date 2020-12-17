use rand::{random, Closed01};
use std::time::Duration;

fn apply_jitter(duration: Duration, jitter: f64) -> Duration {
    let secs = (duration.as_secs() as f64) * jitter;
    let nanos = (duration.subsec_nanos() as f64) * jitter;
    let millis = (secs * 1000f64) + (nanos / 1000000f64);
    Duration::from_millis(millis as u64)
}

pub fn jitter(duration: Duration) -> Duration {
    let Closed01(jitter) = random();
    apply_jitter(duration, jitter)
}

#[test]
fn apply_jitter_quickcheck() {
    extern crate quickcheck;

    #[derive(Clone, Debug)]
    struct ArbitraryJitter(f64);

    impl quickcheck::Arbitrary for ArbitraryJitter {
        fn arbitrary<G: quickcheck::Gen>(g: &mut G) -> Self {
            let Closed01(jitter) = g.gen();
            ArbitraryJitter(jitter)
        }
    }

    fn rounds_correctly(millis: u64, arb_jitter: ArbitraryJitter) -> bool {
        let ArbitraryJitter(jitter) = arb_jitter;

        let millis_with_jitter = ((millis as f64) * jitter) as u64;
        let duration_with_jitter = apply_jitter(Duration::from_millis(millis), jitter);

        duration_with_jitter == Duration::from_millis(millis_with_jitter)
    }

    quickcheck::quickcheck(rounds_correctly as fn(u64, ArbitraryJitter) -> bool)
}
