/**
 * Type System Tests
 * 
 * Tests error types and validation
 */

import { describe, it, expect } from 'vitest';
import {
  VelesDBError,
  ValidationError,
  ConnectionError,
  NotFoundError,
} from '../src/types';

describe('Error Types', () => {
  describe('VelesDBError', () => {
    it('should create error with code', () => {
      const error = new VelesDBError('Test error', 'TEST_CODE');
      expect(error.message).toBe('Test error');
      expect(error.code).toBe('TEST_CODE');
      expect(error.name).toBe('VelesDBError');
    });

    it('should include cause', () => {
      const cause = new Error('Original error');
      const error = new VelesDBError('Wrapped error', 'WRAP', cause);
      expect(error.cause).toBe(cause);
    });
  });

  describe('ValidationError', () => {
    it('should create validation error', () => {
      const error = new ValidationError('Invalid input');
      expect(error.message).toBe('Invalid input');
      expect(error.code).toBe('VALIDATION_ERROR');
      expect(error.name).toBe('ValidationError');
    });

    it('should be instanceof VelesDBError', () => {
      const error = new ValidationError('Test');
      expect(error instanceof VelesDBError).toBe(true);
    });
  });

  describe('ConnectionError', () => {
    it('should create connection error', () => {
      const error = new ConnectionError('Connection failed');
      expect(error.message).toBe('Connection failed');
      expect(error.code).toBe('CONNECTION_ERROR');
      expect(error.name).toBe('ConnectionError');
    });

    it('should include cause', () => {
      const cause = new Error('Network error');
      const error = new ConnectionError('Connection failed', cause);
      expect(error.cause).toBe(cause);
    });
  });

  describe('NotFoundError', () => {
    it('should create not found error', () => {
      const error = new NotFoundError('Collection');
      expect(error.message).toBe('Collection not found');
      expect(error.code).toBe('NOT_FOUND');
      expect(error.name).toBe('NotFoundError');
    });
  });
});

describe('Type Guards', () => {
  it('should allow catching specific error types', () => {
    const handleError = (error: unknown): string => {
      if (error instanceof NotFoundError) {
        return 'not_found';
      }
      if (error instanceof ValidationError) {
        return 'validation';
      }
      if (error instanceof ConnectionError) {
        return 'connection';
      }
      if (error instanceof VelesDBError) {
        return 'velesdb';
      }
      return 'unknown';
    };

    expect(handleError(new NotFoundError('test'))).toBe('not_found');
    expect(handleError(new ValidationError('test'))).toBe('validation');
    expect(handleError(new ConnectionError('test'))).toBe('connection');
    expect(handleError(new VelesDBError('test', 'CODE'))).toBe('velesdb');
    expect(handleError(new Error('test'))).toBe('unknown');
  });
});
