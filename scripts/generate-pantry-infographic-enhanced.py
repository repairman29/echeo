#!/usr/bin/env python3
"""
Generate Enhanced Pantry Infographic PDF
Beautiful visual representation with charts and graphics
"""

import json
import os
from pathlib import Path
from reportlab.lib import colors
from reportlab.lib.pagesizes import letter
from reportlab.lib.units import inch
from reportlab.lib.styles import getSampleStyleSheet, ParagraphStyle
from reportlab.lib.enums import TA_CENTER, TA_LEFT, TA_RIGHT
from reportlab.platypus import SimpleDocTemplate, Paragraph, Spacer, Table, TableStyle, PageBreak, Image
from reportlab.platypus.flowables import HRFlowable, KeepTogether
from reportlab.pdfgen import canvas
from reportlab.graphics.shapes import Drawing, Rect, String
from reportlab.graphics.charts.piecharts import Pie
from reportlab.graphics.charts.barcharts import VerticalBarChart
from reportlab.graphics import renderPDF
from reportlab.graphics.charts.legends import Legend
from reportlab.graphics.widgets.markers import makeMarker
import math

# Beautiful color palette
COLORS = {
    'primary': colors.HexColor('#0EA5E9'),      # Cyan
    'secondary': colors.HexColor('#8B5CF6'),    # Purple
    'accent': colors.HexColor('#F59E0B'),      # Amber
    'success': colors.HexColor('#10B981'),     # Green
    'danger': colors.HexColor('#EF4444'),       # Red
    'dark': colors.HexColor('#1E293B'),         # Slate
    'light': colors.HexColor('#F1F5F9'),       # Light slate
    'gradient_start': colors.HexColor('#0EA5E9'),
    'gradient_end': colors.HexColor('#8B5CF6'),
    'chart_colors': [
        colors.HexColor('#0EA5E9'),  # Cyan
        colors.HexColor('#8B5CF6'),  # Purple
        colors.HexColor('#F59E0B'),  # Amber
        colors.HexColor('#10B981'),  # Green
        colors.HexColor('#EF4444'),  # Red
        colors.HexColor('#EC4899'),  # Pink
        colors.HexColor('#14B8A6'),  # Teal
        colors.HexColor('#F97316'),  # Orange
    ]
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
    """Create a stunning title page"""
    canvas_obj.saveState()
    
    width, height = letter
    
    # Gradient background
    canvas_obj.setFillColor(COLORS['primary'])
    canvas_obj.rect(0, 0, width, height, fill=1)
    
    # Decorative elements
    canvas_obj.setFillColor(colors.white)
    canvas_obj.setStrokeColor(colors.white)
    canvas_obj.setLineWidth(2)
    
    # Title
    canvas_obj.setFillColor(colors.white)
    canvas_obj.setFont("Helvetica-Bold", 56)
    canvas_obj.drawCentredString(width/2, height - 180, "THE PANTRY")
    
    # Subtitle
    canvas_obj.setFont("Helvetica", 20)
    canvas_obj.drawCentredString(width/2, height - 230, "Complete Capability Catalog")
    
    # Main stat - huge number
    canvas_obj.setFont("Helvetica-Bold", 96)
    canvas_obj.setFillColor(COLORS['accent'])
    canvas_obj.drawCentredString(width/2, height/2 + 20, "20,946")
    
    canvas_obj.setFont("Helvetica", 32)
    canvas_obj.setFillColor(colors.white)
    canvas_obj.drawCentredString(width/2, height/2 - 50, "Capabilities")
    
    # Stats row
    canvas_obj.setFont("Helvetica-Bold", 18)
    y_pos = height/2 - 120
    stats = [
        ("56", "Repos"),
        ("44,306", "Files"),
        ("14,448", "Signals")
    ]
    
    for i, (num, label) in enumerate(stats):
        x_pos = width/4 + (i * width/4)
        canvas_obj.drawCentredString(x_pos, y_pos, num)
        canvas_obj.setFont("Helvetica", 12)
        canvas_obj.drawCentredString(x_pos, y_pos - 25, label)
        canvas_obj.setFont("Helvetica-Bold", 18)
    
    # Footer
    canvas_obj.setFont("Helvetica", 12)
    canvas_obj.setFillColor(colors.white)
    canvas_obj.drawCentredString(width/2, 60, "Your Complete Development Inventory")
    canvas_obj.drawCentredString(width/2, 40, "Generated: January 2, 2026")
    
    canvas_obj.restoreState()

def create_summary_page(story, data):
    """Create executive summary with visual stats"""
    summary = data['summary']
    styles = getSampleStyleSheet()
    
    # Title
    title_style = ParagraphStyle(
        'CustomTitle',
        parent=styles['Title'],
        fontSize=36,
        textColor=COLORS['primary'],
        spaceAfter=30,
        alignment=TA_CENTER,
        fontName='Helvetica-Bold'
    )
    story.append(Paragraph("Executive Summary", title_style))
    story.append(Spacer(1, 0.2*inch))
    
    # Create visual stats boxes
    stats_data = [
        ['', 'Value', ''],
        ['📦 Repositories', format_number(summary['totalRepos']), ''],
        ['🌐 Public', format_number(summary['publicRepos']), ''],
        ['🔒 Private', format_number(summary['privateRepos']), ''],
        ['⚡ Capabilities', format_number(summary['totalCapabilities']), ''],
        ['📡 Signals', format_number(summary['totalSignals']), ''],
        ['📄 Files', format_number(summary['totalFiles']), ''],
        ['📊 Average', f"{summary['averageCapabilities']} per repo", ''],
    ]
    
    stats_table = Table(stats_data, colWidths=[2.5*inch, 2*inch, 1.5*inch])
    stats_table.setStyle(TableStyle([
        ('BACKGROUND', (0, 0), (-1, 0), COLORS['primary']),
        ('TEXTCOLOR', (0, 0), (-1, 0), colors.white),
        ('ALIGN', (0, 0), (-1, -1), 'LEFT'),
        ('ALIGN', (1, 0), (1, -1), 'RIGHT'),
        ('FONTNAME', (0, 0), (-1, 0), 'Helvetica-Bold'),
        ('FONTSIZE', (0, 0), (-1, 0), 16),
        ('FONTSIZE', (0, 1), (-1, -1), 12),
        ('BOTTOMPADDING', (0, 0), (-1, 0), 15),
        ('TOPPADDING', (0, 1), (-1, -1), 10),
        ('BOTTOMPADDING', (0, 1), (-1, -1), 10),
        ('BACKGROUND', (0, 1), (-1, -1), colors.white),
        ('GRID', (0, 0), (-1, -1), 1, COLORS['light']),
        ('ROWBACKGROUNDS', (0, 1), (-1, -1), [colors.white, COLORS['light']]),
        ('FONTNAME', (1, 1), (1, -1), 'Helvetica-Bold'),
        ('TEXTCOLOR', (1, 1), (1, -1), COLORS['primary']),
    ]))
    
    story.append(stats_table)
    story.append(Spacer(1, 0.4*inch))
    
    # Revenue potential section
    revenue_style = ParagraphStyle(
        'Revenue',
        parent=styles['Heading2'],
        fontSize=24,
        textColor=COLORS['success'],
        spaceAfter=20,
        alignment=TA_CENTER,
        fontName='Helvetica-Bold'
    )
    story.append(Paragraph("💰 Revenue Potential", revenue_style))
    story.append(Spacer(1, 0.1*inch))
    
    capabilities = summary['totalCapabilities']
    potential_tools = capabilities // 10
    
    revenue_data = [
        ['Estimate', 'Monthly', 'Annual'],
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
        ('FONTSIZE', (0, 0), (-1, 0), 14),
        ('FONTSIZE', (0, 1), (-1, -1), 12),
        ('BOTTOMPADDING', (0, 0), (-1, 0), 15),
        ('TOPPADDING', (0, 1), (-1, -1), 12),
        ('BOTTOMPADDING', (0, 1), (-1, -1), 12),
        ('BACKGROUND', (0, 1), (-1, -1), colors.white),
        ('GRID', (0, 0), (-1, -1), 1, COLORS['light']),
        ('FONTNAME', (1, 1), (2, -1), 'Helvetica-Bold'),
        ('TEXTCOLOR', (1, 1), (2, -1), COLORS['success']),
    ]))
    
    story.append(revenue_table)

def create_top_repos_page(story, repos):
    """Create top repositories with visual ranking"""
    styles = getSampleStyleSheet()
    
    title_style = ParagraphStyle(
        'CustomTitle',
        parent=styles['Title'],
        fontSize=32,
        textColor=COLORS['primary'],
        spaceAfter=25,
        alignment=TA_CENTER,
        fontName='Helvetica-Bold'
    )
    story.append(Paragraph("🏆 Top Repositories", title_style))
    story.append(Spacer(1, 0.2*inch))
    
    # Get top 20 repos
    top_repos = sorted([r for r in repos if r.get('capabilities', 0) > 0], 
                      key=lambda x: x.get('capabilities', 0), reverse=True)[:20]
    
    # Create table with visual indicators
    table_data = [['Rank', 'Repository', 'Capabilities', 'Signals', 'Files']]
    
    for idx, repo in enumerate(top_repos, 1):
        name = repo.get('full_name', 'Unknown')
        if repo.get('private'):
            name = f"🔒 {name}"
        if repo.get('archived'):
            name = f"{name} 📦"
        
        # Truncate long names
        if len(name) > 35:
            name = name[:32] + "..."
        
        table_data.append([
            f"#{idx}",
            name,
            format_number(repo.get('capabilities', 0)),
            format_number(repo.get('signals', 0)),
            format_number(repo.get('filesScanned', 0))
        ])
    
    repo_table = Table(table_data, colWidths=[0.6*inch, 3.2*inch, 1.1*inch, 1*inch, 1*inch])
    repo_table.setStyle(TableStyle([
        ('BACKGROUND', (0, 0), (-1, 0), COLORS['primary']),
        ('TEXTCOLOR', (0, 0), (-1, 0), colors.white),
        ('ALIGN', (0, 0), (-1, -1), 'CENTER'),
        ('ALIGN', (1, 1), (1, -1), 'LEFT'),
        ('FONTNAME', (0, 0), (-1, 0), 'Helvetica-Bold'),
        ('FONTSIZE', (0, 0), (-1, 0), 11),
        ('FONTSIZE', (0, 1), (-1, -1), 9),
        ('BOTTOMPADDING', (0, 0), (-1, 0), 12),
        ('TOPPADDING', (0, 1), (-1, -1), 8),
        ('BOTTOMPADDING', (0, 1), (-1, -1), 8),
        ('BACKGROUND', (0, 1), (-1, -1), colors.white),
        ('GRID', (0, 0), (-1, -1), 1, COLORS['light']),
        ('ROWBACKGROUNDS', (0, 1), (-1, -1), [colors.white, COLORS['light']]),
        # Highlight top 3
        ('BACKGROUND', (0, 1), (-1, 3), COLORS['accent']),
        ('TEXTCOLOR', (0, 1), (-1, 3), colors.white),
        ('FONTNAME', (1, 1), (1, 3), 'Helvetica-Bold'),
    ]))
    
    story.append(repo_table)

def create_distribution_chart(repos):
    """Create a pie chart for capability distribution"""
    drawing = Drawing(400, 300)
    
    # Categorize repos
    mega = sum(1 for r in repos if r.get('capabilities', 0) >= 1000)
    large = sum(1 for r in repos if 500 <= r.get('capabilities', 0) < 1000)
    medium = sum(1 for r in repos if 100 <= r.get('capabilities', 0) < 500)
    small = sum(1 for r in repos if 1 <= r.get('capabilities', 0) < 100)
    empty = sum(1 for r in repos if r.get('capabilities', 0) == 0)
    
    # Create pie chart
    pc = Pie()
    pc.x = 150
    pc.y = 50
    pc.width = 200
    pc.height = 200
    pc.data = [mega, large, medium, small, empty]
    pc.labels = ['Mega (1000+)', 'Large (500-999)', 'Medium (100-499)', 'Small (1-99)', 'Empty (0)']
    pc.slices.strokeWidth = 2
    pc.slices[0].fillColor = COLORS['chart_colors'][0]
    pc.slices[1].fillColor = COLORS['chart_colors'][1]
    pc.slices[2].fillColor = COLORS['chart_colors'][2]
    pc.slices[3].fillColor = COLORS['chart_colors'][3]
    pc.slices[4].fillColor = COLORS['chart_colors'][4]
    
    drawing.add(pc)
    return drawing

def create_capability_bar_chart(repos):
    """Create bar chart for top repos"""
    drawing = Drawing(500, 300)
    
    # Get top 10 repos
    top_repos = sorted([r for r in repos if r.get('capabilities', 0) > 0], 
                      key=lambda x: x.get('capabilities', 0), reverse=True)[:10]
    
    # Prepare data
    capabilities = [r.get('capabilities', 0) for r in top_repos]
    labels = [r.get('name', 'Unknown')[:15] for r in top_repos]
    
    # Create bar chart
    bc = VerticalBarChart()
    bc.x = 50
    bc.y = 50
    bc.width = 400
    bc.height = 200
    bc.data = [capabilities]
    bc.categoryAxis.categoryNames = labels
    bc.bars[0].fillColor = COLORS['primary']
    bc.valueAxis.valueMin = 0
    bc.valueAxis.valueMax = max(capabilities) * 1.1
    
    drawing.add(bc)
    return drawing

def create_distribution_page(story, repos):
    """Create distribution visualization page"""
    styles = getSampleStyleSheet()
    
    title_style = ParagraphStyle(
        'CustomTitle',
        parent=styles['Title'],
        fontSize=32,
        textColor=COLORS['primary'],
        spaceAfter=20,
        alignment=TA_CENTER,
        fontName='Helvetica-Bold'
    )
    story.append(Paragraph("📊 Distribution Analysis", title_style))
    story.append(Spacer(1, 0.2*inch))
    
    # Add pie chart
    pie_chart = create_distribution_chart(repos)
    story.append(pie_chart)
    story.append(Spacer(1, 0.2*inch))
    
    # Distribution table
    mega = sum(1 for r in repos if r.get('capabilities', 0) >= 1000)
    large = sum(1 for r in repos if 500 <= r.get('capabilities', 0) < 1000)
    medium = sum(1 for r in repos if 100 <= r.get('capabilities', 0) < 500)
    small = sum(1 for r in repos if 1 <= r.get('capabilities', 0) < 100)
    empty = sum(1 for r in repos if r.get('capabilities', 0) == 0)
    
    dist_data = [
        ['Category', 'Count', 'Percentage'],
        ['Mega (1000+)', str(mega), f"{(mega/len(repos)*100):.1f}%"],
        ['Large (500-999)', str(large), f"{(large/len(repos)*100):.1f}%"],
        ['Medium (100-499)', str(medium), f"{(medium/len(repos)*100):.1f}%"],
        ['Small (1-99)', str(small), f"{(small/len(repos)*100):.1f}%"],
        ['Empty (0)', str(empty), f"{(empty/len(repos)*100):.1f}%"],
    ]
    
    dist_table = Table(dist_data, colWidths=[2.5*inch, 1.5*inch, 1.5*inch])
    dist_table.setStyle(TableStyle([
        ('BACKGROUND', (0, 0), (-1, 0), COLORS['secondary']),
        ('TEXTCOLOR', (0, 0), (-1, 0), colors.white),
        ('ALIGN', (0, 0), (-1, -1), 'LEFT'),
        ('ALIGN', (1, 0), (2, -1), 'CENTER'),
        ('FONTNAME', (0, 0), (-1, 0), 'Helvetica-Bold'),
        ('FONTSIZE', (0, 0), (-1, 0), 12),
        ('FONTSIZE', (0, 1), (-1, -1), 10),
        ('BOTTOMPADDING', (0, 0), (-1, 0), 12),
        ('TOPPADDING', (0, 1), (-1, -1), 10),
        ('BOTTOMPADDING', (0, 1), (-1, -1), 10),
        ('BACKGROUND', (0, 1), (-1, -1), colors.white),
        ('GRID', (0, 0), (-1, -1), 1, COLORS['light']),
        ('ROWBACKGROUNDS', (0, 1), (-1, -1), [colors.white, COLORS['light']]),
    ]))
    
    story.append(dist_table)
    story.append(Spacer(1, 0.3*inch))
    
    # Private vs Public
    private_count = sum(1 for r in repos if r.get('private'))
    public_count = sum(1 for r in repos if not r.get('private'))
    
    visibility_data = [
        ['Visibility', 'Count', 'Percentage'],
        ['🔒 Private', str(private_count), f"{(private_count/len(repos)*100):.1f}%"],
        ['🌐 Public', str(public_count), f"{(public_count/len(repos)*100):.1f}%"],
    ]
    
    vis_table = Table(visibility_data, colWidths=[2*inch, 2*inch, 2*inch])
    vis_table.setStyle(TableStyle([
        ('BACKGROUND', (0, 0), (-1, 0), COLORS['accent']),
        ('TEXTCOLOR', (0, 0), (-1, 0), colors.white),
        ('ALIGN', (0, 0), (-1, -1), 'CENTER'),
        ('FONTNAME', (0, 0), (-1, 0), 'Helvetica-Bold'),
        ('FONTSIZE', (0, 0), (-1, 0), 12),
        ('FONTSIZE', (0, 1), (-1, -1), 11),
        ('BOTTOMPADDING', (0, 0), (-1, 0), 12),
        ('TOPPADDING', (0, 1), (-1, -1), 12),
        ('BOTTOMPADDING', (0, 1), (-1, -1), 12),
        ('BACKGROUND', (0, 1), (-1, -1), colors.white),
        ('GRID', (0, 0), (-1, -1), 1, COLORS['light']),
    ]))
    
    story.append(vis_table)

def create_insights_page(story, data):
    """Create insights and opportunities page"""
    repos = data['repos']
    summary = data['summary']
    styles = getSampleStyleSheet()
    
    title_style = ParagraphStyle(
        'CustomTitle',
        parent=styles['Title'],
        fontSize=32,
        textColor=COLORS['primary'],
        spaceAfter=20,
        alignment=TA_CENTER,
        fontName='Helvetica-Bold'
    )
    story.append(Paragraph("💡 Key Insights", title_style))
    story.append(Spacer(1, 0.2*inch))
    
    # Top 5 analysis
    top_repos = sorted([r for r in repos if r.get('capabilities', 0) > 0], 
                      key=lambda x: x.get('capabilities', 0), reverse=True)[:5]
    top_5_total = sum(r.get('capabilities', 0) for r in top_repos)
    top_5_percentage = (top_5_total / summary['totalCapabilities']) * 100
    
    insights = [
        ['Insight', 'Details'],
        ['🎯 Top 5 Concentration', f'{top_5_percentage:.1f}% of capabilities in top 5 repos ({format_number(top_5_total)} total)'],
        ['🔒 Hidden Value', f'{summary["privateRepos"]} private repos contain most capabilities'],
        ['🛠️ Product Potential', f'{format_number(summary["totalCapabilities"])} capabilities = ~{format_number(summary["totalCapabilities"]//10)} potential tools'],
        ['💰 Revenue Opportunity', f'${format_number(summary["totalCapabilities"]//10 * 12)}K-${format_number(summary["totalCapabilities"]//10 * 60)}K annual potential'],
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
        ('TOPPADDING', (0, 1), (-1, -1), 10),
        ('BOTTOMPADDING', (0, 1), (-1, -1), 10),
        ('BACKGROUND', (0, 1), (-1, -1), colors.white),
        ('GRID', (0, 0), (-1, -1), 1, COLORS['light']),
        ('ROWBACKGROUNDS', (0, 1), (-1, -1), [colors.white, COLORS['light']]),
    ]))
    
    story.append(insights_table)
    story.append(Spacer(1, 0.3*inch))
    
    # Top 5 repos list
    heading_style = ParagraphStyle(
        'Heading2',
        parent=styles['Heading2'],
        fontSize=18,
        textColor=COLORS['secondary'],
        spaceAfter=10,
        fontName='Helvetica-Bold'
    )
    story.append(Paragraph("🏆 Top 5 Repositories", heading_style))
    story.append(Spacer(1, 0.1*inch))
    
    for idx, repo in enumerate(top_repos, 1):
        name = repo.get('full_name', 'Unknown')
        caps = repo.get('capabilities', 0)
        signals = repo.get('signals', 0)
        files = repo.get('filesScanned', 0)
        
        item_style = ParagraphStyle(
            'ListItem',
            parent=styles['Normal'],
            fontSize=11,
            leftIndent=20,
            spaceAfter=8,
            fontName='Helvetica-Bold'
        )
        story.append(Paragraph(
            f"{idx}. <b>{name}</b> - {format_number(caps)} capabilities, {format_number(signals)} signals, {format_number(files)} files",
            item_style
        ))

def create_charts_page(story, repos):
    """Create page with visual charts"""
    styles = getSampleStyleSheet()
    
    title_style = ParagraphStyle(
        'CustomTitle',
        parent=styles['Title'],
        fontSize=32,
        textColor=COLORS['primary'],
        spaceAfter=20,
        alignment=TA_CENTER,
        fontName='Helvetica-Bold'
    )
    story.append(Paragraph("📈 Visual Analytics", title_style))
    story.append(Spacer(1, 0.2*inch))
    
    # Bar chart
    bar_chart = create_capability_bar_chart(repos)
    story.append(bar_chart)
    story.append(Spacer(1, 0.2*inch))
    
    # Chart description
    desc_style = ParagraphStyle(
        'Description',
        parent=styles['Normal'],
        fontSize=10,
        textColor=colors.grey,
        alignment=TA_CENTER,
        spaceAfter=20
    )
    story.append(Paragraph("Top 10 Repositories by Capability Count", desc_style))

def generate_pdf():
    """Generate the complete enhanced PDF infographic"""
    # Load data
    data = load_catalog_data()
    repos = data['repos']
    summary = data['summary']
    
    # Create PDF
    script_dir = Path(__file__).parent
    output_path = script_dir.parent / 'docs' / 'repo-catalog' / 'THE_PANTRY_INFOGRAPHIC.pdf'
    
    doc = SimpleDocTemplate(
        str(output_path),
        pagesize=letter,
        rightMargin=0.75*inch,
        leftMargin=0.75*inch,
        topMargin=0.75*inch,
        bottomMargin=0.75*inch
    )
    
    story = []
    
    # Page 1: Title (handled by onFirstPage)
    story.append(PageBreak())
    
    # Page 2: Executive Summary
    create_summary_page(story, data)
    story.append(PageBreak())
    
    # Page 3: Top Repos
    create_top_repos_page(story, repos)
    story.append(PageBreak())
    
    # Page 4: Charts
    create_charts_page(story, repos)
    story.append(PageBreak())
    
    # Page 5: Distribution
    create_distribution_page(story, repos)
    story.append(PageBreak())
    
    # Page 6: Insights
    create_insights_page(story, data)
    
    # Build PDF
    doc.build(story, onFirstPage=create_title_page, onLaterPages=lambda c, d: None)
    
    print(f"✅ Enhanced PDF generated: {output_path}")
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

