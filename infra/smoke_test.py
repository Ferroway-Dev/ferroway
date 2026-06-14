#!/usr/bin/env python3
"""
Ferroway infrastructure smoke test.
Run after `docker compose up -d` to verify all services are healthy.

Usage:
    python infra/smoke_test.py
"""

import sys
import time


def check_kafka():
    try:
        from confluent_kafka.admin import AdminClient
        client = AdminClient({"bootstrap.servers": "localhost:9092"})
        metadata = client.list_topics(timeout=5)
        print(f"  ✓ Kafka — connected (KRaft mode, {len(metadata.topics)} topics)")
        return True
    except Exception as e:
        print(f"  ✗ Kafka — {e}")
        return False


def check_postgres():
    try:
        import psycopg2
        conn = psycopg2.connect(
            host="localhost",
            port=5432,
            dbname="ferroway",
            user="ferroway",
            password="ferroway_local",
            connect_timeout=5,
        )
        cur = conn.cursor()
        cur.execute("SELECT extname FROM pg_extension WHERE extname = 'vector';")
        result = cur.fetchone()
        conn.close()
        if result:
            print("  ✓ PostgreSQL — connected (pgvector extension active)")
        else:
            print("  ✗ PostgreSQL — connected but pgvector extension not found")
            return False
        return True
    except Exception as e:
        print(f"  ✗ PostgreSQL — {e}")
        return False


def check_minio():
    try:
        import urllib.request
        req = urllib.request.urlopen("http://localhost:9000/minio/health/live", timeout=5)
        if req.status == 200:
            print("  ✓ MinIO — healthy (S3-compatible object storage)")
            return True
    except Exception as e:
        print(f"  ✗ MinIO — {e}")
        return False


def main():
    print("\nFerroway Infrastructure Smoke Test")
    print("=" * 40)

    checks = [
        ("Kafka (KRaft)", check_kafka),
        ("PostgreSQL + pgvector", check_postgres),
        ("MinIO", check_minio),
    ]

    results = []
    for name, fn in checks:
        print(f"\nChecking {name}...")
        results.append(fn())

    print("\n" + "=" * 40)
    passed = sum(results)
    total = len(results)

    if passed == total:
        print(f"✓ All {total} services healthy — Ferroway infrastructure ready\n")
        sys.exit(0)
    else:
        print(f"✗ {total - passed}/{total} services failed")
        print("  Run: docker compose -f infra/docker-compose.yml up -d")
        print("  Then wait 30s and retry\n")
        sys.exit(1)


if __name__ == "__main__":
    main()
