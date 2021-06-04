
NEAR_ACCOUNT=your_account_here.testnet

build:
	cargo build --target wasm32-unknown-unknown --release

test:
	cargo test -- --nocapture

deploy:
	near dev-deploy --wasmFile target/wasm32-unknown-unknown/release/nft_demo.wasm

metadata:
	near --accountId $(NEAR_ACCOUNT) view `cat neardev/dev-account` nft_metadata

mint:
	near --accountId $(NEAR_ACCOUNT) call `cat neardev/dev-account` nft_mint '{"token_id":"123", "metadata": {"title": "New NFT"}}'

view:
	near --accountId $(NEAR_ACCOUNT) view `cat neardev/dev-account` nft_token '{"token_id":"123"}' 

clean:
	rm -r ./neardev/
