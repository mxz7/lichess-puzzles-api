env "local" {
  dev = "sqlite://file?mode=memory&_fk=1"

  schema {
    src = "file://src/db/schema.sql"
  }

  migration {
    dir = "file://src/db/migrations"
  }
}
