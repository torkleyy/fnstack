extern crate fnstack;

use fnstack::FnStackOnce;

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
        ];

        let mut next = move || heterogeneous.remove(0).unwrap();

        // Increase `value` by 2 and assert it's the result divided by two
        assert_eq!(next().call(2), 3);

        // Square `12`
        assert_eq!(next().call(12), 144);

        // 42 + 3
        assert_eq!(next().call(3), 45);
    }

    assert_eq!(value, 7);
}
