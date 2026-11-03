-- Migration 003: Add topic, article_title, and duration to task_execution_logs
ALTER TABLE task_execution_logs ADD COLUMN topic TEXT;
ALTER TABLE task_execution_logs ADD COLUMN article_title TEXT;
ALTER TABLE task_execution_logs ADD COLUMN duration INTEGER;
