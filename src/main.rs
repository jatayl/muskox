use muskox::bitboard::Bitboard;

fn main() {
	let s = Bitboard::new();
	println!("{}", s.pretty());
	// println!("{:?}", s.get_movers_white());
	// println!("{:?}", s.get_jumpers_white());
	// println!("{:?}", s.fen());

	let s = Bitboard::new_from_fen("B:W18,24,27,28,K10,K15:B12,16,20,K22,K25,K29").unwrap();

	println!("{}", s.pretty());
	println!("{:?}", s.get_movers_white());
	println!("{:?}", s.get_movers_black());
}
