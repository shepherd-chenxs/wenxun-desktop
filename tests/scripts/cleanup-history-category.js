const initSqlJs = require('sql.js');
const fs = require('fs');

const dbPath = 'C:\\Users\\admin\\AppData\\Roaming\\wenxun-desktop\\wenxun.db';

async function cleanupDuplicateHistoryCategory() {
  try {
    console.log('清理重复的历史分类...\n');

    const SQL = await initSqlJs();
    const buffer = fs.readFileSync(dbPath);
    const db = new SQL.Database(buffer);

    // 先删除新创建的历史分类 (category_id: history_001)
    console.log('删除重复的历史分类 (category_id: history_001)...');
    db.run("DELETE FROM topic_categories WHERE category_id = 'history_001'");

    // 验证结果
    console.log('\n清理后的数据:');
    const result = db.exec("SELECT id, category_id, name FROM topic_categories");
    if (result.length > 0) {
      result[0].values.forEach(row => {
        console.log(`  id: ${row[0]}, category_id: ${row[1]}, name: ${row[2]}`);
      });
    }

    console.log('\n重复的历史分类:');
    const duplicateResult = db.exec("SELECT name, COUNT(*) as count FROM topic_categories GROUP BY name HAVING COUNT(*) > 1");
    if (duplicateResult.length > 0) {
      duplicateResult[0].values.forEach(row => {
        console.log(`  name: ${row[0]}, count: ${row[1]}`);
      });
    } else {
      console.log('  无');
    }

    // 保存数据库
    const data = db.export();
    const newBuffer = Buffer.from(data);
    fs.writeFileSync(dbPath, newBuffer);
    console.log('\n数据库已保存');

    db.close();
    console.log('\n清理完成!');
    process.exit(0);
  } catch (error) {
    console.error('失败:', error);
    process.exit(1);
  }
}

cleanupDuplicateHistoryCategory();