extern crate fnstack;

use fnstack::FnStackRef;

fn main() {
    let value = 5;

    {
        let value = &value;
        let mut heterogeneous: Vec<FnStackRef<_, _>> = vec![
            FnStackRef::new(|x| *value / x),
            FnStackRef::new(|x| x * x),
            FnStackRef::new(|x| x + 42),
            FnStackRef::from(Box::new(|x| 2usize * x) as Box<Fn(usize) -> usize>),
        ];

        let mut next = move || heterogeneous.remove(0);

        // 5 / 2
        assert_eq!(next().call(2), 2);

        // Square `12`
        assert_eq!(next().call(12), 144);

        // 42 + 3
        assert_eq!(next().call(3), 45);

        // 2 * 5
        assert_eq!(next().call(5), 10);
    }

    assert_eq!(value, 5);
}
