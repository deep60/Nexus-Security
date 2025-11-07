// MongoDB Initialization Script for Nexus-Security
// This script creates collections and indexes for the threat intelligence platform

db = db.getSiblingDB('nexus_security');

print('Creating Nexus-Security MongoDB collections...');

// ==================== ANALYSIS RESULTS ====================
db.createCollection('analysis_results', {
    validator: {
        $jsonSchema: {
            bsonType: 'object',
            required: ['submission_id', 'engine_id', 'verdict', 'confidence', 'timestamp'],
            properties: {
                submission_id: {
                    bsonType: 'string',
                    description: 'UUID of the submission'
                },
                engine_id: {
                    bsonType: 'string',
                    description: 'ID of the analysis engine'
                },
                verdict: {
                    enum: ['malicious', 'benign', 'suspicious', 'unknown'],
                    description: 'Analysis verdict'
                },
                confidence: {
                    bsonType: 'double',
                    minimum: 0,
                    maximum: 1,
                    description: 'Confidence score (0-1)'
                },
                stake_amount: {
                    bsonType: 'double',
                    minimum: 0,
                    description: 'Amount staked on this verdict'
                },
                timestamp: {
                    bsonType: 'date',
                    description: 'Analysis completion time'
                },
                details: {
                    bsonType: 'object',
                    description: 'Detailed analysis results'
                }
            }
        }
    }
});

db.analysis_results.createIndex({ submission_id: 1 });
db.analysis_results.createIndex({ engine_id: 1 });
db.analysis_results.createIndex({ timestamp: -1 });
db.analysis_results.createIndex({ verdict: 1, confidence: -1 });

print('Created collection: analysis_results');

// ==================== FILE METADATA ====================
db.createCollection('file_metadata', {
    validator: {
        $jsonSchema: {
            bsonType: 'object',
            required: ['file_hash', 'file_type', 'file_size', 'upload_timestamp'],
            properties: {
                file_hash: {
                    bsonType: 'string',
                    description: 'SHA256 hash of the file'
                },
                file_type: {
                    bsonType: 'string',
                    description: 'MIME type or file extension'
                },
                file_size: {
                    bsonType: 'long',
                    description: 'File size in bytes'
                },
                upload_timestamp: {
                    bsonType: 'date'
                },
                storage_path: {
                    bsonType: 'string',
                    description: 'S3/MinIO storage path'
                },
                pe_info: {
                    bsonType: 'object',
                    description: 'PE file information (if applicable)'
                },
                strings: {
                    bsonType: 'array',
                    description: 'Extracted strings from file'
                },
                imports: {
                    bsonType: 'array',
                    description: 'Imported functions/libraries'
                },
                exports: {
                    bsonType: 'array',
                    description: 'Exported functions'
                }
            }
        }
    }
});

db.file_metadata.createIndex({ file_hash: 1 }, { unique: true });
db.file_metadata.createIndex({ file_type: 1 });
db.file_metadata.createIndex({ upload_timestamp: -1 });

print('Created collection: file_metadata');

// ==================== ENGINE STATISTICS ====================
db.createCollection('engine_stats', {
    validator: {
        $jsonSchema: {
            bsonType: 'object',
            required: ['engine_id', 'total_analyses', 'accuracy_rate', 'last_updated'],
            properties: {
                engine_id: {
                    bsonType: 'string'
                },
                engine_name: {
                    bsonType: 'string'
                },
                engine_type: {
                    enum: ['human', 'automated', 'ml', 'static', 'dynamic'],
                    description: 'Type of analysis engine'
                },
                total_analyses: {
                    bsonType: 'int',
                    minimum: 0
                },
                correct_analyses: {
                    bsonType: 'int',
                    minimum: 0
                },
                accuracy_rate: {
                    bsonType: 'double',
                    minimum: 0,
                    maximum: 1
                },
                reputation_score: {
                    bsonType: 'double',
                    minimum: 0,
                    maximum: 100
                },
                total_earnings: {
                    bsonType: 'double'
                },
                total_losses: {
                    bsonType: 'double'
                },
                last_updated: {
                    bsonType: 'date'
                }
            }
        }
    }
});

db.engine_stats.createIndex({ engine_id: 1 }, { unique: true });
db.engine_stats.createIndex({ reputation_score: -1 });
db.engine_stats.createIndex({ accuracy_rate: -1 });

print('Created collection: engine_stats');

// ==================== CONSENSUS RESULTS ====================
db.createCollection('consensus_results', {
    validator: {
        $jsonSchema: {
            bsonType: 'object',
            required: ['submission_id', 'final_verdict', 'confidence_score', 'timestamp'],
            properties: {
                submission_id: {
                    bsonType: 'string'
                },
                final_verdict: {
                    enum: ['malicious', 'benign', 'suspicious', 'unknown'],
                    description: 'Consensus verdict'
                },
                confidence_score: {
                    bsonType: 'double',
                    minimum: 0,
                    maximum: 1
                },
                participating_engines: {
                    bsonType: 'array',
                    description: 'List of engines that participated'
                },
                vote_distribution: {
                    bsonType: 'object',
                    description: 'Distribution of votes'
                },
                timestamp: {
                    bsonType: 'date'
                }
            }
        }
    }
});

db.consensus_results.createIndex({ submission_id: 1 }, { unique: true });
db.consensus_results.createIndex({ final_verdict: 1 });
db.consensus_results.createIndex({ timestamp: -1 });

print('Created collection: consensus_results');

// ==================== THREAT INDICATORS ====================
db.createCollection('threat_indicators', {
    validator: {
        $jsonSchema: {
            bsonType: 'object',
            required: ['indicator_type', 'indicator_value', 'threat_level'],
            properties: {
                indicator_type: {
                    enum: ['ip', 'domain', 'url', 'hash', 'email', 'mutex', 'registry_key'],
                    description: 'Type of IoC'
                },
                indicator_value: {
                    bsonType: 'string'
                },
                threat_level: {
                    enum: ['critical', 'high', 'medium', 'low', 'info'],
                    description: 'Severity level'
                },
                first_seen: {
                    bsonType: 'date'
                },
                last_seen: {
                    bsonType: 'date'
                },
                tags: {
                    bsonType: 'array',
                    description: 'Associated tags (malware family, etc.)'
                },
                source: {
                    bsonType: 'string',
                    description: 'Source of the indicator'
                }
            }
        }
    }
});

db.threat_indicators.createIndex({ indicator_type: 1, indicator_value: 1 }, { unique: true });
db.threat_indicators.createIndex({ threat_level: 1 });
db.threat_indicators.createIndex({ tags: 1 });

print('Created collection: threat_indicators');

// ==================== SANDBOX REPORTS ====================
db.createCollection('sandbox_reports', {
    validator: {
        $jsonSchema: {
            bsonType: 'object',
            required: ['submission_id', 'execution_time', 'report_timestamp'],
            properties: {
                submission_id: {
                    bsonType: 'string'
                },
                execution_time: {
                    bsonType: 'int',
                    description: 'Execution time in seconds'
                },
                network_activity: {
                    bsonType: 'array',
                    description: 'Network connections made'
                },
                file_operations: {
                    bsonType: 'array',
                    description: 'File system operations'
                },
                registry_operations: {
                    bsonType: 'array',
                    description: 'Windows registry changes'
                },
                process_tree: {
                    bsonType: 'object',
                    description: 'Process execution tree'
                },
                screenshots: {
                    bsonType: 'array',
                    description: 'URLs to screenshots'
                },
                report_timestamp: {
                    bsonType: 'date'
                }
            }
        }
    }
});

db.sandbox_reports.createIndex({ submission_id: 1 });
db.sandbox_reports.createIndex({ report_timestamp: -1 });

print('Created collection: sandbox_reports');

print('MongoDB initialization complete!');
print('Collections created: 6');
print('Indexes created: 18');
