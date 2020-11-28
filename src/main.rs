use muskox::bitboard::Bitboard;

fn main() {
	let s = Bitboard::new();
	// println!("{}", s.pretty());
	// println!("{:?}", s.get_movers_white());
	// println!("{:?}", s.get_jumpers_white());
	println!("{:?}", s.fen());

	let s = Bitboard::new_from_fen("B:W18,19,21,23,24,26,29,30,31,32:B1,2,3,4,6,7,9,10,11,12").unwrap();

	println!("{}", s.pretty());
	println!("{:?}", s.fen());
}
