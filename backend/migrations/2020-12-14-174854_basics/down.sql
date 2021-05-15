-- This file should undo anything in `up.sql`
drop trigger jobs_updated_at on jobs;
drop table jobs;

drop table runners;

drop trigger experiments_updated_at on experiments;
drop table experiments;

drop table users;

drop table roles;

drop function update_timestamp();
