INSTALL_DIR=./ts/node_modules/zkwasm-ts-server/src/application
RUNNING_DIR=./ts/node_modules/zkwasm-ts-server
BUILD_ARTIFACTS_DIR=./build-artifacts

default: build

./src/admin.pubkey: ./ts/node_modules/zkwasm-ts-server/src/init_admin.js
	node ./ts/node_modules/zkwasm-ts-server/src/init_admin.js ./src/admin.pubkey

./ts/src/service.js:
	cd ./ts && npx tsc && cd -

build: ./src/admin.pubkey ./ts/src/service.js
	wasm-pack build --release --out-name application --out-dir pkg
	wasm-opt -Oz -o $(INSTALL_DIR)/application_bg.wasm pkg/application_bg.wasm
	cp pkg/application_bg.wasm $(INSTALL_DIR)/application_bg.wasm
	#cp pkg/application.d.ts $(INSTALL_DIR)/application.d.ts
	#cp pkg/application_bg.js $(INSTALL_DIR)/application_bg.js
	cp pkg/application_bg.wasm.d.ts $(INSTALL_DIR)/application_bg.wasm.d.ts
	cd $(RUNNING_DIR) && npx tsc && cd -
	echo "MD5:"
	# 获取 MD5 值并转换为大写
	$(eval MD5_VALUE := $(shell md5sum $(INSTALL_DIR)/application_bg.wasm | awk '{print $$1}' | tr 'a-z' 'A-Z'))
	echo "Calculated MD5: $(MD5_VALUE)"
	# 创建构建产物目录
	mkdir -p $(BUILD_ARTIFACTS_DIR)/application
	# 复制必要的 WASM 文件到构建产物目录
	cp $(INSTALL_DIR)/application_bg.wasm $(BUILD_ARTIFACTS_DIR)/application/
	cp $(INSTALL_DIR)/application_bg.wasm.d.ts $(BUILD_ARTIFACTS_DIR)/application/
	# 记录 MD5 到文件中
	echo "$(MD5_VALUE)" > $(BUILD_ARTIFACTS_DIR)/wasm.md5
	# 直接修改 generate-helm.sh 脚本中的 IMAGE_VALUE
	sed -i 's/^IMAGE_VALUE=.*$$/IMAGE_VALUE="$(MD5_VALUE)"/' scripts/generate-helm.sh
	# 运行 generate-helm.sh 脚本
	chmod +x scripts/generate-helm.sh
	./scripts/generate-helm.sh

clean:
	rm -rf pkg
	rm -rf ./src/admin.pubkey
	rm -rf $(BUILD_ARTIFACTS_DIR)
	rm -rf helm-charts

run:
	node ./ts/src/service.js

deploy:
	docker build --file ./deploy/service.docker -t zkwasm-server . --network=host
