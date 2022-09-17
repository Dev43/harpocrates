prepare:
	@echo "preparing your repo...."
	cargo build

	@echo "fetching submodules"
	git submodule update

	@echo "Compiling ZkSnark circuit"
	./circom/compile.sh

	@echo "Preparing ZkSnark ceremony"

#  WARNING DO NOT DO THIS IN PRODUCTION, THE ENTROPY IS MINIMAL
	@echo "Step 1 of the ceremony"
	echo entropy! | ./circom/ceremony_step1.sh 
#  WARNING DO NOT DO THIS IN PRODUCTION, THE ENTROPY IS MINIMAL
	@echo "Step 2 of the ceremony"
	echo entropy! | ./circom/ceremony_step2.sh