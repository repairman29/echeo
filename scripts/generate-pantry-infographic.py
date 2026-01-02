#!/usr/bin/env python3
"""
Generate Pantry Infographic PDF
Creates a beautiful visual representation of the complete capability catalog
"""

import json
import os
from pathlib import Path
from reportlab.lib import colors
from reportlab.lib.pagesizes import letter, A4
from reportlab.lib.units import inch
from reportlab.lib.styles import getSampleStyleSheet, ParagraphStyle
from reportlab.lib.enums import TA_CENTER, TA_LEFT, TA_RIGHT
from reportlab.platypus import SimpleDocTemplate, Paragraph, Spacer, Table, TableStyle, PageBreak, Image
from reportlab.platypus.flowables import HRFlowable
from reportlab.pdfgen import canvas
from reportlab.graphics.shapes import Drawing, Rect
from reportlab.graphics.charts.piecharts import Pie
from reportlab.graphics.charts.barcharts import VerticalBarChart
from reportlab.graphics import renderPDF
from reportlab.graphics.charts.legends import Legend
import math

# Color palette
COLORS = {
    'primary': colors.HexColor('#0EA5E9'),  # Cyan
    'secondary': colors.HexColor('#8B5CF6'),  # Purple
    'accent': colors.HexColor('#F59E0B'),  # Amber
    'success': colors.HexColor('#10B981'),  # Green
    'danger': colors.HexColor('#EF4444'),  # Red
    'dark': colors.HexColor('#1E293B'),  # Slate
    'light': colors.HexColor('#F1F5F9'),  # Light slate
    'gradient_start': colors.HexColor('#0EA5E9'),
    'gradient_end': colors.HexColor('#8B5CF6'),
}

def load_catalog_data():
    """Load the catalog JSON data"""
    script_dir = Path(__file__).parent
    catalog_path = script_dir.parent / 'docs' / 'repo-catalog' / 'COMPLETE_CATALOG_FIXED.json'
    
    with open(catalog_path, 'r') as f:
        data = json.load(f)
    
    return data

def format_number(num):
    """Format large numbers with commas"""
    return f"{num:,}"

def create_title_page(canvas_obj, doc):
    """Create a beautiful title page"""
    canvas_obj.saveState()
    
    # Background gradient
    width, height = letter
    canvas_obj.setFillColor(COLORS['primary'])
    canvas_obj.rect(0, 0, width, height, fill=1)
    
    # Title
    canvas_obj.setFillColor(colors.white)
    canvas_obj.setFont("Helvetica-Bold", 48)
    canvas_obj.drawCentredString(width/2, height - 200, "THE PANTRY")
    
    # Subtitle
    canvas_obj.setFont("Helvetica", 24)
    canvas_obj.drawCentredString(width/2, height - 250, "Complete Capability Catalog")
    
    # Main stat
    canvas_obj.setFont("Helvetica-Bold", 72)
    canvas_obj.drawCentredString(width/2, height/2, "20,946")
    canvas_obj.setFont("Helvetica", 36)
    canvas_obj.drawCentredString(width/2, height/2 - 60, "Capabilities")
    
    # Footer
    canvas_obj.setFont("Helvetica", 14)
    canvas_obj.drawCentredString(width/2, 100, "Your Complete Development Inventory")
    canvas_obj.drawCentredString(width/2, 80, "56 Repositories • 44,306 Files • 14,448 Signals")
    
    canvas_obj.restoreState()

def create_summary_page(story, data):
    """Create summary statistics page"""
    summary = data['summary']
    
    # Title
    title_style = ParagraphStyle(
        'CustomTitle',
        parent=getSampleStyleSheet()['Title'],
        fontSize=32,
        textColor=COLORS['primary'],
        spaceAfter=30,
        alignment=TA_CENTER
    )
    story.append(Paragraph("Executive Summary", title_style))
    story.append(Spacer(1, 0.3*inch))
    
    # Key stats table
    stats_data = [
        ['Metric', 'Value'],
        ['Total Repositories', format_number(summary['totalRepos'])],
        ['Public Repos', format_number(summary['publicRepos'])],
        ['Private Repos', format_number(summary['privateRepos'])],
        ['Total Capabilities', format_number(summary['totalCapabilities'])],
        ['Total Signals', format_number(summary['totalSignals'])],
        ['Total Files', format_number(summary['totalFiles'])],
        ['Avg Capabilities/Repo', f"{summary['averageCapabilities']}"],
    ]
    
    stats_table = Table(stats_data, colWidths=[3*inch, 2*inch])
    stats_table.setStyle(TableStyle([
        ('BACKGROUND', (0, 0), (-1, 0), COLORS['primary']),
        ('TEXTCOLOR', (0, 0), (-1, 0), colors.white),
        ('ALIGN', (0, 0), (-1, -1), 'LEFT'),
        ('FONTNAME', (0, 0), (-1, 0), 'Helvetica-Bold'),
        ('FONTSIZE', (0, 0), (-1, 0), 14),
        ('BOTTOMPADDING', (0, 0), (-1, 0), 12),
        ('BACKGROUND', (0, 1), (-1, -1), COLORS['light']),
        ('GRID', (0, 0), (-1, -1), 1, colors.grey),
        ('ROWBACKGROUNDS', (0, 1), (-1, -1), [colors.white, COLORS['light']]),
    ]))
    
    story.append(stats_table)
    story.append(Spacer(1, 0.5*inch))
    
    # Revenue potential
    revenue_style = ParagraphStyle(
        'Revenue',
        parent=getSampleStyleSheet()['Heading2'],
        fontSize=20,
        textColor=COLORS['success'],
        spaceAfter=15,
    )
    story.append(Paragraph("Revenue Potential", revenue_style))
    
    capabilities = summary['totalCapabilities']
    potential_tools = capabilities // 10  # Rough estimate
    
    revenue_data = [
        ['Estimate', 'Monthly Revenue', 'Annual Revenue'],
        ['Conservative', f'${potential_tools * 1:,}K', f'${potential_tools * 12:,}K'],
        ['Moderate', f'${potential_tools * 2:,}K', f'${potential_tools * 24:,}K'],
        ['Aggressive', f'${potential_tools * 5:,}K', f'${potential_tools * 60:,}K'],
    ]
    
    revenue_table = Table(revenue_data, colWidths=[2*inch, 2*inch, 2*inch])
    revenue_table.setStyle(TableStyle([
        ('BACKGROUND', (0, 0), (-1, 0), COLORS['success']),
        ('TEXTCOLOR', (0, 0), (-1, 0), colors.white),
        ('ALIGN', (0, 0), (-1, -1), 'CENTER'),
        ('FONTNAME', (0, 0), (-1, 0), 'Helvetica-Bold'),
        ('FONTSIZE', (0, 0), (-1, 0), 12),
        ('BOTTOMPADDING', (0, 0), (-1, 0), 12),
        ('BACKGROUND', (0, 1), (-1, -1), colors.white),
        ('GRID', (0, 0), (-1, -1), 1, colors.grey),
    ]))
    
    story.append(revenue_table)

def create_top_repos_page(story, repos):
    """Create top repositories visualization"""
    # Title
    title_style = ParagraphStyle(
        'CustomTitle',
        parent=getSampleStyleSheet()['Title'],
        fontSize=28,
        textColor=COLORS['primary'],
        spaceAfter=20,
        alignment=TA_CENTER
    )
    story.append(Paragraph("Top Repositories by Capabilities", title_style))
    story.append(Spacer(1, 0.2*inch))
    
    # Get top 15 repos
    top_repos = sorted([r for r in repos if r.get('capabilities', 0) > 0], 
                      key=lambda x: x.get('capabilities', 0), reverse=True)[:15]
    
    # Create table
    table_data = [['Rank', 'Repository', 'Capabilities', 'Signals', 'Files']]
    
    for idx, repo in enumerate(top_repos, 1):
        name = repo.get('full_name', 'Unknown')
        if repo.get('private'):
            name += ' 🔒'
        if repo.get('archived'):
            name += ' 📦'
        
        table_data.append([
            str(idx),
            name,
            format_number(repo.get('capabilities', 0)),
            format_number(repo.get('signals', 0)),
            format_number(repo.get('filesScanned', 0))
        ])
    
    repo_table = Table(table_data, colWidths=[0.5*inch, 3*inch, 1*inch, 1*inch, 1*inch])
    repo_table.setStyle(TableStyle([
        ('BACKGROUND', (0, 0), (-1, 0), COLORS['primary']),
        ('TEXTCOLOR', (0, 0), (-1, 0), colors.white),
        ('ALIGN', (0, 0), (-1, -1), 'CENTER'),
        ('FONTNAME', (0, 0), (-1, 0), 'Helvetica-Bold'),
        ('FONTSIZE', (0, 0), (-1, 0), 10),
        ('FONTSIZE', (0, 1), (-1, -1), 8),
        ('BOTTOMPADDING', (0, 0), (-1, 0), 12),
        ('BACKGROUND', (0, 1), (-1, -1), colors.white),
        ('GRID', (0, 0), (-1, -1), 1, colors.grey),
        ('ROWBACKGROUNDS', (0, 1), (-1, -1), [colors.white, COLORS['light']]),
        ('ALIGN', (1, 1), (1, -1), 'LEFT'),  # Left align repo names
    ]))
    
    story.append(repo_table)

def create_distribution_page(story, repos):
    """Create capability distribution visualization"""
    title_style = ParagraphStyle(
        'CustomTitle',
        parent=getSampleStyleSheet()['Title'],
        fontSize=28,
        textColor=COLORS['primary'],
        spaceAfter=20,
        alignment=TA_CENTER
    )
    story.append(Paragraph("Capability Distribution", title_style))
    story.append(Spacer(1, 0.2*inch))
    
    # Categorize repos
    mega = sum(1 for r in repos if r.get('capabilities', 0) >= 1000)
    large = sum(1 for r in repos if 500 <= r.get('capabilities', 0) < 1000)
    medium = sum(1 for r in repos if 100 <= r.get('capabilities', 0) < 500)
    small = sum(1 for r in repos if 1 <= r.get('capabilities', 0) < 100)
    empty = sum(1 for r in repos if r.get('capabilities', 0) == 0)
    
    # Distribution table
    dist_data = [
        ['Category', 'Count', 'Description'],
        ['Mega (1000+)', str(mega), 'Massive repositories with 1000+ capabilities'],
        ['Large (500-999)', str(large), 'Large repositories with 500-999 capabilities'],
        ['Medium (100-499)', str(medium), 'Medium repositories with 100-499 capabilities'],
        ['Small (1-99)', str(small), 'Small repositories with 1-99 capabilities'],
        ['Empty (0)', str(empty), 'Repositories with no code or capabilities'],
    ]
    
    dist_table = Table(dist_data, colWidths=[2*inch, 1*inch, 3*inch])
    dist_table.setStyle(TableStyle([
        ('BACKGROUND', (0, 0), (-1, 0), COLORS['secondary']),
        ('TEXTCOLOR', (0, 0), (-1, 0), colors.white),
        ('ALIGN', (0, 0), (-1, -1), 'LEFT'),
        ('FONTNAME', (0, 0), (-1, 0), 'Helvetica-Bold'),
        ('FONTSIZE', (0, 0), (-1, 0), 12),
        ('BOTTOMPADDING', (0, 0), (-1, 0), 12),
        ('BACKGROUND', (0, 1), (-1, -1), colors.white),
        ('GRID', (0, 0), (-1, -1), 1, colors.grey),
        ('ROWBACKGROUNDS', (0, 1), (-1, -1), [colors.white, COLORS['light']]),
    ]))
    
    story.append(dist_table)
    story.append(Spacer(1, 0.3*inch))
    
    # Private vs Public
    private_count = sum(1 for r in repos if r.get('private'))
    public_count = sum(1 for r in repos if not r.get('private'))
    
    visibility_data = [
        ['Visibility', 'Count', 'Percentage'],
        ['Private', str(private_count), f"{(private_count/len(repos)*100):.1f}%"],
        ['Public', str(public_count), f"{(public_count/len(repos)*100):.1f}%"],
    ]
    
    vis_table = Table(visibility_data, colWidths=[2*inch, 1.5*inch, 1.5*inch])
    vis_table.setStyle(TableStyle([
        ('BACKGROUND', (0, 0), (-1, 0), COLORS['accent']),
        ('TEXTCOLOR', (0, 0), (-1, 0), colors.white),
        ('ALIGN', (0, 0), (-1, -1), 'CENTER'),
        ('FONTNAME', (0, 0), (-1, 0), 'Helvetica-Bold'),
        ('FONTSIZE', (0, 0), (-1, 0), 12),
        ('BOTTOMPADDING', (0, 0), (-1, 0), 12),
        ('BACKGROUND', (0, 1), (-1, -1), colors.white),
        ('GRID', (0, 0), (-1, -1), 1, colors.grey),
    ]))
    
    story.append(vis_table)

def create_insights_page(story, data):
    """Create insights and opportunities page"""
    repos = data['repos']
    summary = data['summary']
    
    title_style = ParagraphStyle(
        'CustomTitle',
        parent=getSampleStyleSheet()['Title'],
        fontSize=28,
        textColor=COLORS['primary'],
        spaceAfter=20,
        alignment=TA_CENTER
    )
    story.append(Paragraph("Key Insights & Opportunities", title_style))
    story.append(Spacer(1, 0.2*inch))
    
    # Top 5 repos contain 85% of capabilities
    top_repos = sorted([r for r in repos if r.get('capabilities', 0) > 0], 
                      key=lambda x: x.get('capabilities', 0), reverse=True)[:5]
    top_5_total = sum(r.get('capabilities', 0) for r in top_repos)
    top_5_percentage = (top_5_total / summary['totalCapabilities']) * 100
    
    insights = [
        ['Insight', 'Details'],
        ['Top 5 Repos', f'Contain {top_5_percentage:.1f}% of all capabilities ({format_number(top_5_total)} total)'],
        ['Hidden Value', f'{summary["privateRepos"]} private repos contain most capabilities'],
        ['Product Potential', f'{summary["totalCapabilities"]} capabilities = ~{summary["totalCapabilities"]//10} potential tools'],
        ['Revenue Opportunity', f'${summary["totalCapabilities"]//10 * 12}K-${summary["totalCapabilities"]//10 * 60}K annual potential'],
    ]
    
    insights_table = Table(insights, colWidths=[2.5*inch, 3.5*inch])
    insights_table.setStyle(TableStyle([
        ('BACKGROUND', (0, 0), (-1, 0), COLORS['dark']),
        ('TEXTCOLOR', (0, 0), (-1, 0), colors.white),
        ('ALIGN', (0, 0), (-1, -1), 'LEFT'),
        ('FONTNAME', (0, 0), (-1, 0), 'Helvetica-Bold'),
        ('FONTSIZE', (0, 0), (-1, 0), 12),
        ('FONTSIZE', (0, 1), (-1, -1), 10),
        ('BOTTOMPADDING', (0, 0), (-1, 0), 12),
        ('BACKGROUND', (0, 1), (-1, -1), colors.white),
        ('GRID', (0, 0), (-1, -1), 1, colors.grey),
        ('ROWBACKGROUNDS', (0, 1), (-1, -1), [colors.white, COLORS['light']]),
    ]))
    
    story.append(insights_table)
    story.append(Spacer(1, 0.3*inch))
    
    # Top repos list
    story.append(Paragraph("Top 5 Repositories", 
                          ParagraphStyle('Heading2', parent=getSampleStyleSheet()['Heading2'], 
                                        fontSize=16, textColor=COLORS['secondary'])))
    story.append(Spacer(1, 0.1*inch))
    
    top_list = []
    for idx, repo in enumerate(top_repos, 1):
        name = repo.get('full_name', 'Unknown')
        caps = repo.get('capabilities', 0)
        top_list.append(f"{idx}. {name}: {format_number(caps)} capabilities")
    
    for item in top_list:
        story.append(Paragraph(f"• {item}", 
                              ParagraphStyle('CustomBody', parent=getSampleStyleSheet()['Normal'], 
                                            fontSize=10, leftIndent=20)))
        story.append(Spacer(1, 0.05*inch))

def generate_pdf():
    """Generate the complete PDF infographic"""
    # Load data
    data = load_catalog_data()
    repos = data['repos']
    summary = data['summary']
    
    # Create PDF
    script_dir = Path(__file__).parent
    output_path = script_dir.parent / 'docs' / 'repo-catalog' / 'THE_PANTRY_INFOGRAPHIC.pdf'
    
    doc = SimpleDocTemplate(str(output_path), pagesize=letter,
                           rightMargin=0.75*inch, leftMargin=0.75*inch,
                           topMargin=0.75*inch, bottomMargin=0.75*inch)
    
    story = []
    
    # Page 1: Title (handled by onFirstPage)
    story.append(PageBreak())
    
    # Page 2: Summary
    create_summary_page(story, data)
    story.append(PageBreak())
    
    # Page 3: Top Repos
    create_top_repos_page(story, repos)
    story.append(PageBreak())
    
    # Page 4: Distribution
    create_distribution_page(story, repos)
    story.append(PageBreak())
    
    # Page 5: Insights
    create_insights_page(story, data)
    
    # Build PDF
    doc.build(story, onFirstPage=create_title_page, onLaterPages=lambda c, d: None)
    
    print(f"✅ PDF generated: {output_path}")
    return output_path

if __name__ == '__main__':
    try:
        output_path = generate_pdf()
        print(f"\n🍳 Your Pantry Infographic is ready!")
        print(f"   Location: {output_path}\n")
    except Exception as e:
        print(f"❌ Error generating PDF: {e}")
        import traceback
        traceback.print_exc()

