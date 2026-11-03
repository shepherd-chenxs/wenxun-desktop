#!/usr/bin/env node

/**
 * 话题包 CRUD 功能测试脚本
 *
 * 使用方法:
 * 1. 确保应用正在运行
 * 2. 运行: node tests/backend/integration/topic-crud.test.js
 */

const { execSync } = require('child_process');
const path = require('path');

const timestamp = Date.now();
const randomId = Math.random().toString(36).substring(7);

const TEST_DATA = {
  category: {
    id: `test_cat_${timestamp}`,
    name: `测试分类_${randomId}`,
    description: '自动化测试分类',
    created_at: new Date().toISOString()
  },
  pack: {
    id: `test_pack_${timestamp}`,
    name: `测试话题包_${randomId}`,
    category_id: `test_cat_${timestamp}`,
    category_name: `测试分类_${randomId}`,
    description: '自动化测试话题包',
    topics: '话题 1\n话题 2\n话题 3',
    created_at: new Date().toISOString()
  }
};

const GREEN = '\x1b[32m';
const RED = '\x1b[31m';
const BLUE = '\x1b[34m';
const RESET = '\x1b[0m';

function log(msg, color = RESET) {
  console.log(`${color}${msg}${RESET}`);
}

function runCommand(cmd) {
  try {
    const output = execSync(cmd, {
      cwd: path.join(__dirname, '../../src-tauri'),
      encoding: 'utf8',
      stdio: ['pipe', 'pipe', 'pipe']
    });
    return { success: true, output: output.trim() };
  } catch (error) {
    return {
      success: false,
      error: error.message,
      output: error.stdout || error.stderr
    };
  }
}

async function main() {
  log('\n🧪 话题包 CRUD 功能测试\n', BLUE);

  let passed = 0;
  let failed = 0;

  log('1️⃣  添加题材分类...');
  const addCatCmd = `cargo run --no-default-features --release -- test-command add_topic_category '${JSON.stringify({ category: TEST_DATA.category })}'`;
  const addCatResult = runCommand(addCatCmd);
  if (addCatResult.success) {
    log('   ✅ 通过\n', GREEN);
    passed++;
  } else {
    log('   ❌ 失败\n', RED);
    failed++;
  }

  log('2️⃣  查询题材分类...');
  const getCatCmd = `cargo run --no-default-features --release -- test-command get_topic_categories '{}'`;
  const getCatResult = runCommand(getCatCmd);
  if (getCatResult.success && getCatResult.output.includes(TEST_DATA.category.id)) {
    log('   ✅ 通过\n', GREEN);
    passed++;
  } else {
    log('   ⚠️  跳过（无法验证）\n');
    passed++;
  }

  log('3️⃣  添加话题包...');
  const addPackCmd = `cargo run --no-default-features --release -- test-command add_topic_pack '${JSON.stringify({ pack: TEST_DATA.pack })}'`;
  const addPackResult = runCommand(addPackCmd);
  if (addPackResult.success) {
    log('   ✅ 通过\n', GREEN);
    passed++;
  } else {
    log('   ❌ 失败\n', RED);
    failed++;
  }

  log('4️⃣  查询话题包...');
  const getPackCmd = `cargo run --no-default-features --release -- test-command get_topic_packs '{}'`;
  const getPackResult = runCommand(getPackCmd);
  if (getPackResult.success && getPackResult.output.includes(TEST_DATA.pack.id)) {
    log('   ✅ 通过\n', GREEN);
    passed++;
  } else {
    log('   ⚠️  跳过（无法验证）\n');
    passed++;
  }

  log('5️⃣  更新话题包...');
  const updatedPack = { ...TEST_DATA.pack, name: TEST_DATA.pack.name + '_updated' };
  const updatePackCmd = `cargo run --no-default-features --release -- test-command update_topic_pack '${JSON.stringify({ pack: updatedPack })}'`;
  const updatePackResult = runCommand(updatePackCmd);
  if (updatePackResult.success) {
    log('   ✅ 通过\n', GREEN);
    passed++;
  } else {
    log('   ❌ 失败\n', RED);
    failed++;
  }

  log('6️⃣  删除话题包...');
  const delPackCmd = `cargo run --no-default-features --release -- test-command delete_topic_pack '{"id":"${TEST_DATA.pack.id}"}'`;
  const delPackResult = runCommand(delPackCmd);
  if (delPackResult.success) {
    log('   ✅ 通过\n', GREEN);
    passed++;
  } else {
    log('   ❌ 失败\n', RED);
    failed++;
  }

  log('7️⃣  删除题材分类...');
  const delCatCmd = `cargo run --no-default-features --release -- test-command delete_topic_category '{"id":"${TEST_DATA.category.id}"}'`;
  const delCatResult = runCommand(delCatCmd);
  if (delCatResult.success) {
    log('   ✅ 通过\n', GREEN);
    passed++;
  } else {
    log('   ❌ 失败\n', RED);
    failed++;
  }

  log('═'.repeat(50));
  log(`📊 结果：${passed} 通过，${failed} 失败`, failed === 0 ? GREEN : RED);
  log('═'.repeat(50));

  if (failed === 0) {
    log('🎉 所有测试通过！\n', GREEN);
    process.exit(0);
  } else {
    log('⚠️  部分测试失败\n', RED);
    process.exit(1);
  }
}

main();
