prepare:
	@echo "preparing your repo...."
	cd fhe_contract && cargo build

	@echo "fetching submodules"
	git submodule init && git submodule update

	@echo "Compiling ZkSnark circuit"
	cd fhe_contract && cd circom  &&  ./compile.sh

	@echo "Preparing ZkSnark ceremony"

#  WARNING DO NOT DO THIS IN PRODUCTION, THE ENTROPY IS MINIMAL
	@echo "Step 1 of the ceremony"
	cd fhe_contract && (echo entropy! |  ./circom/ceremony_step1.sh )
#  WARNING DO NOT DO THIS IN PRODUCTION, THE ENTROPY IS MINIMAL
	@echo "Step 2 of the ceremony"
	 cd fhe_contract && (echo entropy! | ./circom/ceremony_step2.sh)

	 @echo "You are now ready, a great welcome from Harpocrates!"