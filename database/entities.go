package database

import (
	"time"
)

type User struct {
	ID        int64 `gorm:"primaryKey; notNull; autoIncrement:false"`
	ChatID    int64 `gorm:"notNull"`
	OmsCard   int64
	BirthDate *time.Time
}
