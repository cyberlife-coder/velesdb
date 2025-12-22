/**
 * VelesDB Client - Unified interface for all backends
 */

import type {
  VelesDBConfig,
  CollectionConfig,
  Collection,
  VectorDocument,
  SearchOptions,
  SearchResult,
  IVelesDBBackend,
} from './types';
import { ValidationError } from './types';
import { WasmBackend } from './backends/wasm';
import { RestBackend } from './backends/rest';

/**
 * VelesDB Client
 * 
 * Provides a unified interface for interacting with VelesDB
 * using either WASM (browser/Node.js) or REST API backends.
 * 
 * @example
 * ```typescript
 * const db = new VelesDB({ backend: 'wasm' });
 * await db.init();
 * 
 * await db.createCollection('embeddings', { dimension: 768, metric: 'cosine' });
 * await db.insert('embeddings', { id: 'doc1', vector: [...], payload: { title: 'Hello' } });
 * 
 * const results = await db.search('embeddings', queryVector, { k: 5 });
 * ```
 */
export class VelesDB {
  private readonly config: VelesDBConfig;
  private backend: IVelesDBBackend;
  private initialized = false;

  /**
   * Create a new VelesDB client
   * 
   * @param config - Client configuration
   * @throws {ValidationError} If configuration is invalid
   */
  constructor(config: VelesDBConfig) {
    this.validateConfig(config);
    this.config = config;
    this.backend = this.createBackend(config);
  }

  private validateConfig(config: VelesDBConfig): void {
    if (!config.backend) {
      throw new ValidationError('Backend type is required');
    }

    if (config.backend !== 'wasm' && config.backend !== 'rest') {
      throw new ValidationError(`Invalid backend type: ${config.backend}. Use 'wasm' or 'rest'`);
    }

    if (config.backend === 'rest' && !config.url) {
      throw new ValidationError('URL is required for REST backend');
    }
  }

  private createBackend(config: VelesDBConfig): IVelesDBBackend {
    switch (config.backend) {
      case 'wasm':
        return new WasmBackend();
      case 'rest':
        return new RestBackend(config.url!, config.apiKey, config.timeout);
      default:
        throw new ValidationError(`Unknown backend: ${config.backend}`);
    }
  }

  /**
   * Initialize the client
   * Must be called before any other operations
   */
  async init(): Promise<void> {
    if (this.initialized) {
      return;
    }
    await this.backend.init();
    this.initialized = true;
  }

  /**
   * Check if client is initialized
   */
  isInitialized(): boolean {
    return this.initialized;
  }

  private ensureInitialized(): void {
    if (!this.initialized) {
      throw new ValidationError('Client not initialized. Call init() first.');
    }
  }

  /**
   * Create a new collection
   * 
   * @param name - Collection name
   * @param config - Collection configuration
   */
  async createCollection(name: string, config: CollectionConfig): Promise<void> {
    this.ensureInitialized();
    
    if (!name || typeof name !== 'string') {
      throw new ValidationError('Collection name must be a non-empty string');
    }
    
    if (!config.dimension || config.dimension <= 0) {
      throw new ValidationError('Dimension must be a positive integer');
    }

    await this.backend.createCollection(name, config);
  }

  /**
   * Delete a collection
   * 
   * @param name - Collection name
   */
  async deleteCollection(name: string): Promise<void> {
    this.ensureInitialized();
    await this.backend.deleteCollection(name);
  }

  /**
   * Get collection information
   * 
   * @param name - Collection name
   * @returns Collection info or null if not found
   */
  async getCollection(name: string): Promise<Collection | null> {
    this.ensureInitialized();
    return this.backend.getCollection(name);
  }

  /**
   * List all collections
   * 
   * @returns Array of collections
   */
  async listCollections(): Promise<Collection[]> {
    this.ensureInitialized();
    return this.backend.listCollections();
  }

  /**
   * Insert a vector document
   * 
   * @param collection - Collection name
   * @param doc - Document to insert
   */
  async insert(collection: string, doc: VectorDocument): Promise<void> {
    this.ensureInitialized();
    this.validateDocument(doc);
    await this.backend.insert(collection, doc);
  }

  /**
   * Insert multiple vector documents
   * 
   * @param collection - Collection name
   * @param docs - Documents to insert
   */
  async insertBatch(collection: string, docs: VectorDocument[]): Promise<void> {
    this.ensureInitialized();
    
    if (!Array.isArray(docs)) {
      throw new ValidationError('Documents must be an array');
    }

    for (const doc of docs) {
      this.validateDocument(doc);
    }

    await this.backend.insertBatch(collection, docs);
  }

  private validateDocument(doc: VectorDocument): void {
    if (doc.id === undefined || doc.id === null) {
      throw new ValidationError('Document ID is required');
    }

    if (!doc.vector) {
      throw new ValidationError('Document vector is required');
    }

    if (!Array.isArray(doc.vector) && !(doc.vector instanceof Float32Array)) {
      throw new ValidationError('Vector must be an array or Float32Array');
    }
  }

  /**
   * Search for similar vectors
   * 
   * @param collection - Collection name
   * @param query - Query vector
   * @param options - Search options
   * @returns Search results sorted by relevance
   */
  async search(
    collection: string,
    query: number[] | Float32Array,
    options?: SearchOptions
  ): Promise<SearchResult[]> {
    this.ensureInitialized();

    if (!query || (!Array.isArray(query) && !(query instanceof Float32Array))) {
      throw new ValidationError('Query must be an array or Float32Array');
    }

    return this.backend.search(collection, query, options);
  }

  /**
   * Delete a vector by ID
   * 
   * @param collection - Collection name
   * @param id - Document ID
   * @returns true if deleted, false if not found
   */
  async delete(collection: string, id: string | number): Promise<boolean> {
    this.ensureInitialized();
    return this.backend.delete(collection, id);
  }

  /**
   * Get a vector by ID
   * 
   * @param collection - Collection name
   * @param id - Document ID
   * @returns Document or null if not found
   */
  async get(collection: string, id: string | number): Promise<VectorDocument | null> {
    this.ensureInitialized();
    return this.backend.get(collection, id);
  }

  /**
   * Close the client and release resources
   */
  async close(): Promise<void> {
    if (this.initialized) {
      await this.backend.close();
      this.initialized = false;
    }
  }

  /**
   * Get the current backend type
   */
  get backendType(): string {
    return this.config.backend;
  }
}
