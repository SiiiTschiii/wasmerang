#!/usr/bin/env python3
"""
README Link Checker - Ensures all README files are properly linked from the root README.

This script:
1. Finds all README.md files in the repository
2. Checks that each one (except root) is referenced from the root README
3. Validates that the links actually work
4. Reports any orphaned README files
"""

import os
import re
import sys
from pathlib import Path

def find_readme_files(root_dir):
    """Find all README.md files in the repository."""
    readme_files = []
    for root, dirs, files in os.walk(root_dir):
        # Skip hidden directories and build directories
        dirs[:] = [d for d in dirs if not d.startswith('.') and d != 'target']
        
        for file in files:
            if file.lower() == 'readme.md':
                rel_path = os.path.relpath(os.path.join(root, file), root_dir)
                readme_files.append(rel_path)
    
    return sorted(readme_files)

def extract_links_from_readme(readme_path):
    """Extract all relative links from a README file."""
    try:
        with open(readme_path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        # Find markdown links: [text](link) and direct references
        markdown_links = re.findall(r'\[.*?\]\(([^)]+)\)', content)
        direct_refs = re.findall(r'(?:see|check|refer to|found in)\s+([a-zA-Z0-9_/-]+\.md)', content, re.IGNORECASE)
        
        # Filter for local README links
        readme_links = []
        for link in markdown_links + direct_refs:
            if 'readme.md' in link.lower() or link.endswith('/README.md'):
                readme_links.append(link)
        
        return readme_links
    except Exception as e:
        print(f"Error reading {readme_path}: {e}")
        return []

def check_readme_links():
    """Main function to check README links."""
    root_dir = Path.cwd()
    root_readme = root_dir / 'README.md'
    
    print("🔍 Checking README file linkage...")
    print(f"Root directory: {root_dir}")
    
    # Find all README files
    readme_files = find_readme_files(root_dir)
    print(f"\n📄 Found {len(readme_files)} README files:")
    for readme in readme_files:
        print(f"  - {readme}")
    
    if not root_readme.exists():
        print("❌ ERROR: Root README.md not found!")
        return False
    
    # Extract links from root README
    root_links = extract_links_from_readme(root_readme)
    print(f"\n🔗 Links found in root README: {root_links}")
    
    # Check which READMEs are referenced
    referenced_readmes = set()
    for link in root_links:
        # Normalize the link path
        normalized = link.strip('./')
        if normalized in readme_files:
            referenced_readmes.add(normalized)
    
    # Find orphaned READMEs (not linked from root)
    all_readmes = set(readme_files)
    all_readmes.discard('README.md')  # Don't check root against itself
    
    orphaned = all_readmes - referenced_readmes
    
    # Report results
    print(f"\n✅ Referenced READMEs ({len(referenced_readmes)}):")
    for readme in sorted(referenced_readmes):
        print(f"  - {readme}")
    
    if orphaned:
        print(f"\n⚠️  Orphaned READMEs ({len(orphaned)}):")
        for readme in sorted(orphaned):
            print(f"  - {readme}")
        print("\n💡 Consider adding links to these READMEs in the root README.md")
        return False
    else:
        print(f"\n🎉 All README files are properly linked!")
        return True

if __name__ == '__main__':
    success = check_readme_links()
    sys.exit(0 if success else 1)
