#!/usr/bin/env sh

ldif=/home/george/Documents/progs/lightweight-ldap/newentry.ldif

ldapadd -H ldap://127.0.0.1:8000 -D "Dexter McClary"
