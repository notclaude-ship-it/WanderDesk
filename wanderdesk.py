#!/usr/bin/env python3
"""WanderDesk: explorable Linux application launcher prototype."""
import sys, os, math, random, subprocess, configparser, shlex
from pathlib import Path
from app_catalog import discover_apps
from PyQt6.QtCore import Qt, QTimer, QRectF, QPointF
from PyQt6.QtGui import QColor, QPainter, QPen, QBrush, QFont, QLinearGradient
from PyQt6.QtWidgets import QApplication, QWidget, QVBoxLayout, QHBoxLayout, QLabel, QPushButton, QComboBox, QCheckBox, QSlider, QStackedWidget, QFrame
ROOT=Path(__file__).resolve().parent
def apps():
    return discover_apps(limit=90)
class World(QWidget):
    def __init__(self,back,settings):
        super().__init__(); self.back=back; self.settings=settings; self.setFocusPolicy(Qt.FocusPolicy.StrongFocus)
        self.catalog=apps(); random.seed(42)
        self.objects=[(name,cmd,random.uniform(-38,38),random.uniform(-38,38)) for name,cmd in self.catalog]
        self.x=self.y=0.; self.angle=0.; self.keys=set(); self.arm=[]; self.selected=None; self.notice='Approche une application, puis appuie sur E'; self.ticks=0
        self.timer=QTimer(self); self.timer.timeout.connect(self.step); self.timer.start(16)
    def step(self):
        speed=self.settings['speed']*.075; a=self.angle
        if Qt.Key.Key_W in self.keys or Qt.Key.Key_Z in self.keys: self.x+=math.sin(a)*speed; self.y+=math.cos(a)*speed
        if Qt.Key.Key_S in self.keys: self.x-=math.sin(a)*speed; self.y-=math.cos(a)*speed
        if Qt.Key.Key_A in self.keys or Qt.Key.Key_Q in self.keys: self.x-=math.cos(a)*speed; self.y+=math.sin(a)*speed
        if Qt.Key.Key_D in self.keys: self.x+=math.cos(a)*speed; self.y-=math.sin(a)*speed
        if Qt.Key.Key_Left in self.keys: self.angle-=.035
        if Qt.Key.Key_Right in self.keys: self.angle+=.035
        nearest=sorted(((math.hypot(px-self.x,py-self.y),i) for i,(_,_,px,py) in enumerate(self.objects)))
        self.selected=nearest[0][1] if nearest and nearest[0][0]<3.5 else None
        self.ticks+=1; self.update()
    def keyPressEvent(self,e):
        k=e.key()
        if k==Qt.Key.Key_E and self.selected is not None:
            item=self.objects[self.selected]
            if item[0] not in [a[0] for a in self.arm]: self.arm.append(item); self.notice=item[0]+' attachée au bras'
            else: self.arm=[a for a in self.arm if a[0]!=item[0]]; self.notice=item[0]+' détachée'
        elif k==Qt.Key.Key_F and self.selected is not None: self.launch(self.objects[self.selected])
        elif k==Qt.Key.Key_Escape: self.keys.clear(); self.back()
        elif Qt.Key.Key_1<=k<=Qt.Key.Key_9:
            i=k-Qt.Key.Key_1
            if i<len(self.arm): self.launch(self.arm[i])
        else: self.keys.add(k)
    def keyReleaseEvent(self,e): self.keys.discard(e.key())
    def launch(self,item):
        cmd=item[1]
        import re
        cmd=re.sub(r'%[fFuUdDnNickvm]','',cmd)
        try: subprocess.Popen(shlex.split(cmd),start_new_session=True); self.notice='Lancement : '+item[0]
        except Exception as ex: self.notice='Erreur : '+str(ex)
    def project(self,px,py,w,h,eye=0):
        dx=px-self.x-eye*math.cos(self.angle); dy=py-self.y+eye*math.sin(self.angle)
        forward=dx*math.sin(self.angle)+dy*math.cos(self.angle)
        side=dx*math.cos(self.angle)-dy*math.sin(self.angle)
        if forward<.35: return None
        scale=min(h*1.0/forward,1200)
        return w/2+side*scale,h*.52+scale*.38,scale,forward
    def scene(self,p,w,h,eye=0):
        p.fillRect(QRectF(0,0,w,h),QColor('#090f20'))
        g=QLinearGradient(0,0,0,h*.56);g.setColorAt(0,QColor('#101d42'));g.setColorAt(1,QColor('#25445a'));p.fillRect(QRectF(0,0,w,h*.56),g)
        g=QLinearGradient(0,h*.5,0,h);g.setColorAt(0,QColor('#172c3c'));g.setColorAt(1,QColor('#070d19'));p.fillRect(QRectF(0,h*.5,w,h*.5),g)
        p.setPen(QPen(QColor(50,110,135,85),1))
        for n in range(-50,51,2):
            for coords in [((n,-50),(n,50)),((-50,n),(50,n))]:
                a=self.project(*coords[0],w,h,eye);b=self.project(*coords[1],w,h,eye)
                if a and b: p.drawLine(QPointF(a[0],a[1]),QPointF(b[0],b[1]))
        visible=[]
        for i,(name,cmd,x,y) in enumerate(self.objects):
            v=self.project(x,y,w,h,eye)
            if v and v[3]<65: visible.append((v[3],i,name,v))
        for _,i,name,(sx,sy,sc,dist) in sorted(visible,reverse=True):
            size=max(12,min(130,sc*.95)); r=QRectF(sx-size*.55,sy-size*1.25,size*1.1,size*.9)
            p.setPen(QPen(QColor('#72f7f0') if i==self.selected else QColor('#477bba'),2))
            p.setBrush(QColor('#18364d') if i!=self.selected else QColor('#245e69'));p.drawRoundedRect(r,8,8)
            p.setPen(QColor('#e3ffff'));f=p.font();f.setPixelSize(max(9,min(16,int(size*.14))));p.setFont(f)
            p.drawText(r.adjusted(4,3,-4,-3),Qt.AlignmentFlag.AlignCenter,name[:28])
            p.setPen(QPen(QColor('#39d9e6'),2));p.drawLine(QPointF(sx,sy-size*.35),QPointF(sx,sy))
        p.setPen(QPen(QColor('#e2faff'),2));p.drawLine(QPointF(w/2-8,h/2),QPointF(w/2+8,h/2));p.drawLine(QPointF(w/2,h/2-8),QPointF(w/2,h/2+8))
    def paintEvent(self,e):
        p=QPainter(self);p.setRenderHint(QPainter.RenderHint.Antialiasing)
        w=self.width();h=self.height()
        if self.settings['vr']:
            self.scene(p,w/2,h,-.12);p.save();p.translate(w/2,0);self.scene(p,w/2,h,.12);p.restore()
            p.setPen(QColor('#5c8a9c'));p.drawLine(w//2,0,w//2,h)
        else: self.scene(p,w,h)
        p.setPen(QColor('#ffffff'));f=QFont('Sans',11);p.setFont(f)
        p.drawText(22,32,'WANDERDESK   //   EXPLORATION')
        p.drawText(22,55,'ZQSD/WASD déplacer · ← → tourner · E attacher · F ouvrir · Échap menu')
        if self.selected is not None: p.drawText(22,80,'À portée : '+self.objects[self.selected][0]+'   [E] BRAS   [F] OUVRIR')
        p.drawText(22,h-25,self.notice)
        width=260; x=w-width-20
        p.setBrush(QColor(9,19,35,225));p.setPen(QColor('#53b9c8'));p.drawRoundedRect(QRectF(x,18,width,44+len(self.arm)*29),12,12)
        p.setPen(QColor('#ffffff'));p.drawText(x+14,43,'◈  BRAS / APPLICATIONS')
        for i,item in enumerate(self.arm): p.drawText(x+14,72+i*29,str(i+1)+'  '+item[0][:27])
        # Large touch controls, also usable with a mouse
        p.setPen(QPen(QColor(114,247,240,190),2))
        p.setBrush(QColor(10,25,44,205))
        controls=self.touch_controls(w,h)
        for label,rect,key in controls:
            p.drawRoundedRect(rect,14,14)
            p.setPen(QColor('#e5f7ff'))
            p.setFont(QFont('Sans',16 if len(label)<3 else 10,QFont.Weight.Bold))
            p.drawText(rect,Qt.AlignmentFlag.AlignCenter,label)
            p.setPen(QPen(QColor(114,247,240,190),2));p.setBrush(QColor(10,25,44,205))
    def touch_controls(self,w,h):
        s=max(46,min(68,int(min(w,h)*.085))); gap=8
        left=18; bottom=h-s-22; cx=left+s+gap; cy=bottom-s-gap
        right=w-18-s; turn_y=bottom-s-gap
        return [
            ('▲',QRectF(cx,cy-s-gap,s,s),Qt.Key.Key_W),
            ('◀',QRectF(left,cy,s,s),Qt.Key.Key_A),
            ('▼',QRectF(cx,cy,s,s),Qt.Key.Key_S),
            ('▶',QRectF(cx+s+gap,cy,s,s),Qt.Key.Key_D),
            ('↶',QRectF(right,turn_y,s,s),Qt.Key.Key_Left),
            ('↷',QRectF(right,turn_y+s+gap,s,s),Qt.Key.Key_Right),
            ('E  ATTACHER',QRectF(right-s-12,turn_y-s-gap-8,s+12,s), 'E'),
            ('F  OUVRIR',QRectF(right-s-12,turn_y-2*(s+gap)-8,s+12,s), 'F'),
            ('MENU',QRectF(w/2-52, h-62,104,42),'ESC'),
        ]
    def mousePressEvent(self,e):
        pos=e.position(); w=self.width(); h=self.height()
        for label,rect,action in self.touch_controls(w,h):
            if rect.contains(pos):
                if isinstance(action, Qt.Key):
                    self.keys.add(action); self._touch_key=action
                elif action=='E' and self.selected is not None:
                    self.keyPressEvent(type('KeyEvent',(),{'key':lambda self:Qt.Key.Key_E})())
                elif action=='F' and self.selected is not None: self.launch(self.objects[self.selected])
                elif action=='ESC': self.keys.clear(); self.back()
                self.update(); e.accept(); return
        # Tap an attached application in the arm panel to launch it
        width=260; x=w-width-20
        if x<=pos.x()<=w-20 and 50<=pos.y()<=50+len(self.arm)*29:
            i=int((pos.y()-50)//29)
            if 0<=i<len(self.arm): self.launch(self.arm[i])
        super().mousePressEvent(e)
    def mouseReleaseEvent(self,e):
        if hasattr(self,'_touch_key'):
            self.keys.discard(self._touch_key); del self._touch_key
        super().mouseReleaseEvent(e)
class Menu(QWidget):
    def __init__(self,start,settings):
        super().__init__();self.setStyleSheet('QWidget{background-color:#0b1325;color:#e5f7ff;font-size:15px} QLabel{background-color:transparent;border:0} QCheckBox{background-color:transparent;spacing:10px} QSlider{background-color:transparent} QComboBox{background-color:#19334c;color:#e5f7ff;border:1px solid #41889a;border-radius:7px;padding:7px} QComboBox QAbstractItemView{background-color:#13243a;color:#e5f7ff;selection-background-color:#246178} QPushButton{background-color:#19334c;color:#e5f7ff;border:1px solid #41889a;border-radius:12px;padding:18px;min-height:28px} QPushButton:hover{background-color:#246178} QPushButton#play{background-color:#087e83;color:white;font-size:23px;font-weight:bold;padding:18px} QStackedWidget{background-color:#0b1325}')
        root=QVBoxLayout(self);root.setContentsMargins(90,55,90,55);root.setSpacing(20)
        hero=QLabel('WANDERDESK');hero.setObjectName('hero');root.addWidget(hero)
        root.addWidget(QLabel('TON BUREAU EST UN MONDE. Explore, récupère et lance tes applications Linux.'))
        root.addSpacing(20);root.addWidget(QLabel('PARAMÈTRES DE SESSION'))
        vr=QCheckBox('Vue VR stéréoscopique expérimentale (écran partagé)');vr.setChecked(settings['vr']);vr.toggled.connect(lambda v:settings.update(vr=v));root.addWidget(vr)
        root.addWidget(QLabel('Vitesse de déplacement'))
        slider=QSlider(Qt.Orientation.Horizontal);slider.setRange(1,15);slider.setValue(settings['speed']);slider.valueChanged.connect(lambda v:settings.update(speed=v));root.addWidget(slider)
        root.addWidget(QLabel('Affichage : fenêtré ou plein écran'))
        combo=QComboBox();combo.addItems(['Fenêtré','Plein écran']);combo.setCurrentIndex(int(settings['fullscreen']));combo.currentIndexChanged.connect(lambda v:settings.update(fullscreen=bool(v)));root.addWidget(combo)
        root.addWidget(QLabel('Mode actuel : lanceur 3D expérimental. Les applications s’ouvrent dans la session graphique hôte.'))
        root.addStretch(1);play=QPushButton('▶  JOUER');play.setObjectName('play');play.clicked.connect(start);root.addWidget(play)
class Main(QStackedWidget):
    def __init__(self):
        super().__init__();self.settings={'vr':False,'speed':5,'fullscreen':'--fullscreen' in sys.argv}
        self.menu=Menu(self.play,self.settings);self.world=World(self.pause,self.settings)
        self.addWidget(self.menu);self.addWidget(self.world);self.setWindowTitle('WanderDesk');self.resize(1280,800);self.setMinimumSize(850,540)
    def play(self):
        self.setCurrentWidget(self.world)
        if self.settings['fullscreen']:self.showFullScreen()
        else:self.showNormal()
        self.world.setFocus()
    def pause(self):self.showNormal();self.setCurrentWidget(self.menu)
if __name__=='__main__':
    app=QApplication(sys.argv);app.setStyleSheet('QWidget{background-color:#0b1325;color:#e5f7ff} QToolTip{background-color:#19334c;color:#e5f7ff;border:1px solid #41889a}');w=Main();w.setStyleSheet('QStackedWidget{background-color:#0b1325}');w.show();sys.exit(app.exec())