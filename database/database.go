package database

import (
	"gorm.io/driver/sqlite"
	"gorm.io/gorm"
)

func ConnectToDb(path string, config gorm.Config) (*gorm.DB, error) {
	db, err := gorm.Open(sqlite.Open(path), &config)
	if err != nil {
		return nil, err
	}
	return db, nil
}
