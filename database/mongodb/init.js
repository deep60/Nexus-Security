// MongoDB Initialization Entry Point
// This script runs when MongoDB container starts for the first time
// It creates the database, collections, and indexes

// Switch to admin database for initial setup
db = db.getSiblingDB('admin');

// Create application database user with appropriate permissions
db.createUser({
    user: 'verdyx_admin',
    pwd: process.env.MONGODB_PASSWORD || 'verdyx_secure_password',
    roles: [
        {
            role: 'readWrite',
            db: 'verdyx'
        },
        {
            role: 'dbAdmin',
            db: 'verdyx'
        }
    ]
});

print('Created database user: verdyx_admin');

// Switch to application database
db = db.getSiblingDB('verdyx');

// Load and execute the main initialization script
load('/docker-entrypoint-initdb.d/init/init-db.js');

print('MongoDB initialization complete!');
print('Database: verdyx');
print('User: verdyx_admin');
print('Ready for connections...');
