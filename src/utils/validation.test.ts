import { validateTitle, validateContent } from './validation';

describe('Validation utilities', () => {
  describe('validateTitle', () => {
    it('should return valid for non-empty title', () => {
      const result = validateTitle('Test Title');
      expect(result.valid).toBe(true);
      expect(result.error).toBeUndefined();
    });

    it('should return invalid for empty title', () => {
      const result = validateTitle('');
      expect(result.valid).toBe(false);
      expect(result.error).toBe('标题不能为空');
    });

    it('should return invalid for title with only whitespace', () => {
      const result = validateTitle('   ');
      expect(result.valid).toBe(false);
      expect(result.error).toBe('标题不能为空');
    });

    it('should return valid for title with leading/trailing whitespace', () => {
      const result = validateTitle('  Test Title  ');
      expect(result.valid).toBe(true);
      expect(result.error).toBeUndefined();
    });

    it('should return invalid for title longer than 200 characters', () => {
      const longTitle = 'a'.repeat(201);
      const result = validateTitle(longTitle);
      expect(result.valid).toBe(false);
      expect(result.error).toBe('标题长度不能超过200个字符');
    });

    it('should return invalid for title with dangerous characters', () => {
      const result = validateTitle('<script>alert("xss")</script>');
      expect(result.valid).toBe(false);
      expect(result.error).toBe('标题包含非法字符');
    });
  });

  describe('validateContent', () => {
    it('should return valid for non-empty content', () => {
      const result = validateContent('Test content');
      expect(result.valid).toBe(true);
      expect(result.error).toBeUndefined();
    });

    it('should return invalid for empty content', () => {
      const result = validateContent('');
      expect(result.valid).toBe(false);
      expect(result.error).toBe('内容不能为空');
    });

    it('should return invalid for content with only whitespace', () => {
      const result = validateContent('   ');
      expect(result.valid).toBe(false);
      expect(result.error).toBe('内容不能为空');
    });

    it('should return valid for content with leading/trailing whitespace', () => {
      const result = validateContent('  Test content  ');
      expect(result.valid).toBe(true);
      expect(result.error).toBeUndefined();
    });

    it('should return invalid for content longer than 50000 characters', () => {
      const longContent = 'a'.repeat(50001);
      const result = validateContent(longContent);
      expect(result.valid).toBe(false);
      expect(result.error).toBe('内容长度不能超过50000个字符');
    });
  });
});
