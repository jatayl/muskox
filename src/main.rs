use muskox::bitboard::Bitboard;

fn main() {
	let s = Bitboard::new();
	println!("{}", s.pretty());
	println!("{:?}", s.get_movers_white());
	println!("{:?}", s.get_jumpers_white());
}
