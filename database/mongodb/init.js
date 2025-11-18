// MongoDB Initialization Entry Point
// This script runs when MongoDB container starts for the first time
// It creates the database, collections, and indexes

// Switch to admin database for initial setup
db = db.getSiblingDB('admin');

// Create application database user with appropriate permissions
db.createUser({
    user: 'nexus_admin',
    pwd: process.env.MONGODB_PASSWORD || 'nexus_secure_password',
    roles: [
        {
            role: 'readWrite',
            db: 'nexus_security'
        },
        {
            role: 'dbAdmin',
            db: 'nexus_security'
        }
    ]
});

print('Created database user: nexus_admin');

// Switch to application database
db = db.getSiblingDB('nexus_security');

// Load and execute the main initialization script
load('/docker-entrypoint-initdb.d/init/init-db.js');

print('MongoDB initialization complete!');
print('Database: nexus_security');
print('User: nexus_admin');
print('Ready for connections...');
