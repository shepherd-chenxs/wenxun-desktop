const initSqlJs = require('sql.js');
const fs = require('fs');

const dbPath = 'C:\\Users\\admin\\AppData\\Roaming\\wenxun-desktop\\wenxun.db';

async function cleanupDuplicateHistory() {
  try {
    console.log('清理重复的历史分类数据...\n');

    const SQL = await initSqlJs();
    const buffer = fs.readFileSync(dbPath);
    const db = new SQL.Database(buffer);
    console.log('已加载数据库\n');

    // 查询所有历史分类
    const categories = db.exec("SELECT id, category_id, name FROM topic_categories WHERE name = '历史'");
    console.log('现有的历史分类:');
    if (categories.length > 0) {
      categories[0].values.forEach(row => {
        console.log(`  id: ${row[0]}, category_id: ${row[1]}, name: ${row[2]}`);
      });
    }

    // 删除重复的历史分类，只保留 id 最小的那个
    console.log('\n删除重复的历史分类...');
    db.run(`
      DELETE FROM topic_categories
      WHERE name = '历史' AND id NOT IN (
        SELECT MIN(id) FROM topic_categories WHERE name = '历史'
      )
    `);
    console.log('重复的历史分类已删除');

    // 查询所有历史博主
    const bloggers = db.exec("SELECT id, blogger_id, name, topic_category_name FROM virtual_bloggers WHERE topic_category_name = '历史'");
    console.log('\n现有的历史博主:');
    if (bloggers.length > 0) {
      bloggers[0].values.forEach(row => {
        console.log(`  id: ${row[0]}, blogger_id: ${row[1]}, name: ${row[2]}, category: ${row[3]}`);
      });
    }

    // 删除重复的历史博主，只保留 id 最小的那个
    console.log('\n删除重复的历史博主...');
    db.run(`
      DELETE FROM virtual_bloggers
      WHERE topic_category_name = '历史' AND id NOT IN (
        SELECT MIN(id) FROM virtual_bloggers WHERE topic_category_name = '历史'
      )
    `);
    console.log('重复的历史博主已删除');

    // 保存数据库
    const data = db.export();
    const newBuffer = Buffer.from(data);
    fs.writeFileSync(dbPath, newBuffer);
    console.log('\n数据库已保存');

    // 验证结果
    console.log('\n验证清理结果:');
    const finalCategories = db.exec("SELECT id, category_id, name FROM topic_categories WHERE name = '历史'");
    console.log('历史分类:');
    if (finalCategories.length > 0 && finalCategories[0].values.length > 0) {
      finalCategories[0].values.forEach(row => {
        console.log(`  id: ${row[0]}, category_id: ${row[1]}, name: ${row[2]}`);
      });
    } else {
      console.log('  无');
    }

    const finalBloggers = db.exec("SELECT id, blogger_id, name, topic_category_name FROM virtual_bloggers WHERE topic_category_name = '历史'");
    console.log('历史博主:');
    if (finalBloggers.length > 0 && finalBloggers[0].values.length > 0) {
      finalBloggers[0].values.forEach(row => {
        console.log(`  id: ${row[0]}, blogger_id: ${row[1]}, name: ${row[2]}, category: ${row[3]}`);
      });
    } else {
      console.log('  无');
    }

    db.close();
    console.log('\n清理完成!');
    process.exit(0);
  } catch (error) {
    console.error('清理失败:', error);
    process.exit(1);
  }
}

cleanupDuplicateHistory();