#![warn(clippy::large_data_pass_by_val)]

#[derive(Copy, Clone)]
struct A {
    x: [u64; 0x200],
}

struct B {
    x: [u64; 0x200],
}

trait C {
    fn good(&self, arg: &A);
    fn bad(&self, arg: A);
}

impl C for B {
    fn good(&self, arg: &A) {
        unimplemented!()
    }
    fn bad(&self, arg: A) {
        unimplemented!()
    }
}

impl B {
    fn do_bad(&self, arg: A) {
        unimplemented!()
    }
    fn do_good(&self, arg: &A) {
        unimplemented!()
    }
}

fn pass_by_val(arg: A) {
    let sum = arg.x.iter().sum::<u64>();
    println!("{}", sum);
}

fn pass_by_ref(arg: &A) {
    let sum = arg.x.iter().sum::<u64>();
    println!("{}", sum);
}

fn main() {
    let a = A { x: [0; 0x200] };
    pass_by_val(a);
    pass_by_ref(&a);
    let b = B { x: [0; 0x200] };
    b.do_bad(a);
    b.good(&a);
    b.bad(a);
}
