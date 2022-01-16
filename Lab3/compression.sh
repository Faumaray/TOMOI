./Generator --input inp --output msg --range 1024 --count 1000000
read -p "Repair errors?[true,false]: " r
./noise-resistant_coder --input msg --output res
./noisy-pipe --input res --prob 0.1
if [ $r == true ] 
then
	./noise-resistant_decoder --input res.dmg --output out -r 
	./Comparator --one-input msg --two-input out
else
	./noise-resistant_decoder --input res.dmg --output out
fi

