#ifndef VIEWPORTWIDGET_H
#define VIEWPORTWIDGET_H

#include <QWidget>
#include <QtOpenGLWidgets/QOpenGLWidget>

class ViewPortWidget : public QWidget
{
    Q_OBJECT
    struct ViewPortWidgetPrivate;
    ViewPortWidgetPrivate *_p;
public:
    explicit ViewPortWidget(QWidget *parent = nullptr);
    virtual ~ViewPortWidget();

signals:
};

#endif // VIEWPORTWIDGET_H
