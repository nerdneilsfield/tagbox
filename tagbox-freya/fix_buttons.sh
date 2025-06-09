#!/bin/bash

# 修复advanced_search.rs
sed -i 's/background: "rgb(0, 0, 0, 50)"/background: "rgba(0, 0, 0, 0.5)"/g' src/components/advanced_search.rs
sed -i 's/shadow: "0 4 20 0 rgb(0, 0, 0, 30)"/shadow: "0 4 20 0 rgb(200, 200, 200)"/g' src/components/advanced_search.rs

# 替换所有Button为CustomButton
find src -name "*.rs" -type f -exec sed -i 's/Button {/CustomButton {/g' {} \;
find src -name "*.rs" -type f -exec sed -i 's/label { "\([^"]*\)" }/text: "\1",/g' {} \;

echo "修复完成"