namespace dotnet;

partial class Form1
{
    private System.ComponentModel.IContainer components = null;

    protected override void Dispose(bool disposing)
    {
        if (disposing && (components != null))
        {
            components.Dispose();
        }
        base.Dispose(disposing);
    }

    #region Windows Form Designer generated code

    private void InitializeComponent()
    {
        this.SuspendLayout();
        // 
        // Form1
        // 
        this.ClientSize = new System.Drawing.Size(1024, 700);
        this.Name = "Form1";
        this.Text = "Dotnet (Laragon Native Replacement)";
        this.StartPosition = System.Windows.Forms.FormStartPosition.CenterScreen;
        this.ResumeLayout(false);
    }

    #endregion
}
