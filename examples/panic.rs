fn main() {
	mayerror::install();
	throw();
}

fn throw() {
	panic!("something went wrong ...");
}
