extern crate fnstack;

use fnstack::{FnBox, FnStackOnce};

fn main() {
    let mut value = 5;

    {
        let value = &mut value;
        let mut heterogeneous: Vec<FnStackOnce<_, _>> = vec![
            FnStackOnce::new(|x| {
                *value += x;

                *value / x
            }),
            FnStackOnce::new(|x| x * x),
            FnStackOnce::new(|x| x + 42),
            FnStackOnce::from(Box::new(|x| x * 2) as Box<FnBox<_, _>>)
        ];

        let mut next = move || heterogeneous.remove(0);

        // Increase `value` by 2 and assert it's the result divided by two
        assert_eq!(next().call(2), 3);

        // Square `12`
        assert_eq!(next().call(12), 144);

        // 42 + 3
        assert_eq!(next().call(3), 45);

        // 3 * 2
        assert_eq!(next().call(3), 6);
    }

    assert_eq!(value, 7);
}
