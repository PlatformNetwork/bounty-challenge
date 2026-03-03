--- a/src/utils/helper.js
+++ b/src/utils/helper.js
@@ -10,7 +10,7 @@ function processData(input) {
 
   // 旧逻辑：直接返回原始数组
-  return input.filter(item => item.active);
+  // 新逻辑：过滤并转换数据格式
+  return input
+    .filter(item => item.active)
+    .map(item => ({
+      id: item.id,
+      name: item.name.toUpperCase(),
+      status: 'PROCESSED'
+    }));
 }
 
 export default { processData };